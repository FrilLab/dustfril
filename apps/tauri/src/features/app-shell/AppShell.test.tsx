import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AppShell } from './AppShell';
import {
  analyzeWorkspace,
  clearActivityHistory,
  executeCleanup,
  loadActivityHistory,
  refreshStorageVolume,
} from '../../lib/tauri';

vi.mock('../../lib/tauri', () => ({
  analyzeWorkspace: vi.fn(),
  chooseWorkspaceFolder: vi.fn(),
  defaultRoot: vi.fn().mockResolvedValue('/workspace'),
  executeCleanup: vi.fn(),
  refreshStorageVolume: vi.fn(),
  loadActivityHistory: vi.fn().mockResolvedValue([]),
  clearActivityHistory: vi.fn().mockResolvedValue(undefined),
}));

const analyzedArtifact = {
  path: '/workspace/dustfril/target',
  ecosystem: 'Rust' as const,
  project: {
    root: '/workspace/dustfril',
    displayName: 'dustfril',
    ecosystem: 'Rust' as const,
  },
  sizeBytes: 11 * 1024 ** 3,
  lastModifiedMs: Date.UTC(2026, 8, 1),
  ageDays: 30,
  recommendation: 'Keep' as const,
};

const analyzedNodeArtifact = {
  ...analyzedArtifact,
  path: '/workspace/dustfril/node_modules',
  ecosystem: 'Node' as const,
  project: {
    ...analyzedArtifact.project,
    ecosystem: 'Node' as const,
  },
  sizeBytes: 7 * 1024 ** 3,
  recommendation: 'SafeToClean' as const,
};

const historyEntry = {
  id: 'scan-1',
  timestampMs: Date.UTC(2026, 8, 3, 4, 5),
  kind: 'Scan' as const,
  result: {
    success: true,
    details: { path: '/workspace', artifacts: 1, size: 1024 },
  },
};

describe('AppShell Overview navigation', () => {
  afterEach(() => vi.clearAllMocks());

  it.each([
    ['Rust', false],
    ['Node.js', false],
    ['Java', false],
    ['Cache', true],
    ['Dependencies', false],
    ['Artifact History', true],
    ['Activity', false],
    ['Supply Chain', true],
    ['GitHub Actions', true],
    ['Executable Integrity', true],
  ])('navigates to the %s module without starting another operation', async (title, planned) => {
    render(<AppShell />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Analyze Workspace' })).toBeEnabled());

    fireEvent.click(screen.getByRole('button', { name: title }));

    expect(screen.getByRole('button', { name: title })).toHaveAttribute('aria-current', 'page');
    if (planned) {
      expect(screen.getByRole('heading', { name: `${title} is planned` })).toBeInTheDocument();
    }
    expect(analyzeWorkspace).not.toHaveBeenCalled();
  });

  it('opens the exact Overview artifact in Workspace without selecting it', async () => {
    vi.mocked(analyzeWorkspace).mockResolvedValue({
      analysis: {
        artifacts: [analyzedArtifact],
        totalSizeBytes: analyzedArtifact.sizeBytes,
      },
      cleanupPlan: {
        analysisId: 'analysis-1',
        candidates: [
          {
            path: analyzedArtifact.path,
            ecosystem: analyzedArtifact.ecosystem,
            project: analyzedArtifact.project,
            sizeBytes: analyzedArtifact.sizeBytes,
            ageDays: analyzedArtifact.ageDays,
            recommendation: analyzedArtifact.recommendation,
            selectedByDefault: false,
          },
        ],
        reclaimableSizeBytes: 0,
      },
      storageSummary: {
        status: 'available',
        totalBytes: 512 * 1024 ** 3,
        usedBytes: 318 * 1024 ** 3,
        availableBytes: 194 * 1024 ** 3,
        detectedDevelopmentBytes: 11 * 1024 ** 3,
        detectedSharePercent: 3.4591194968553455,
        partial: false,
        warnings: [],
        recommendedBytes: 0,
        scopePath: '/workspace/dustfril',
        categories: ['Rust'],
      },
    });

    render(<AppShell />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Analyze Workspace' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Analyze Workspace' }));
    await waitFor(() =>
      expect(document.querySelector('.workspace-summary-strip')).toHaveTextContent(
        '1 artifact · 11 GB total',
      ),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Overview' }));
    expect(screen.queryByRole('button', { name: 'Cleanup' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^History/ }));
    expect(screen.queryByRole('button', { name: 'Cleanup' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Workspace/ }));
    expect(screen.getByRole('button', { name: 'Cleanup' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Overview' }));
    fireEvent.click(screen.getByRole('button', { name: 'Inspect dustfril target' }));

    expect(screen.getByRole('button', { name: /^Workspace/ })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('complementary', { name: 'Artifact inspector' })).toBeInTheDocument();
    expect(screen.getAllByText('/workspace/dustfril/target').length).toBeGreaterThan(0);
    expect(screen.getByRole('checkbox')).not.toBeChecked();
  });

  it('clears activity history only after confirmation and updates the sidebar count', async () => {
    vi.mocked(loadActivityHistory).mockResolvedValueOnce([historyEntry]);

    render(<AppShell />);
    await waitFor(() => expect(screen.getByRole('button', { name: /^History/ })).toHaveTextContent('1'));
    fireEvent.click(screen.getByRole('button', { name: /^History/ }));

    fireEvent.click(screen.getByRole('button', { name: 'Clear History' }));
    fireEvent.click(screen.getByRole('dialog').querySelector('button') as HTMLButtonElement);
    expect(clearActivityHistory).not.toHaveBeenCalled();
    expect(screen.getByRole('row', { name: /Inspect Scan activity/ })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Clear History' }));
    const dialog = screen.getByRole('dialog');
    fireEvent.click(dialog.querySelectorAll('button')[1] as HTMLButtonElement);

    await waitFor(() => expect(screen.getByRole('button', { name: /^History/ })).toHaveTextContent('0'));
    expect(clearActivityHistory).toHaveBeenCalledOnce();
    expect(screen.getByText('No activity yet')).toBeInTheDocument();
  });

  it('confirms and executes only the selected ecosystem artifacts', async () => {
    vi.mocked(analyzeWorkspace).mockResolvedValue({
      analysis: {
        artifacts: [analyzedArtifact, analyzedNodeArtifact],
        totalSizeBytes: analyzedArtifact.sizeBytes + analyzedNodeArtifact.sizeBytes,
      },
      cleanupPlan: {
        analysisId: 'analysis-1',
        candidates: [
          {
            path: analyzedArtifact.path,
            ecosystem: analyzedArtifact.ecosystem,
            project: analyzedArtifact.project,
            sizeBytes: analyzedArtifact.sizeBytes,
            ageDays: analyzedArtifact.ageDays,
            recommendation: analyzedArtifact.recommendation,
            selectedByDefault: true,
          },
          {
            path: analyzedNodeArtifact.path,
            ecosystem: analyzedNodeArtifact.ecosystem,
            project: analyzedNodeArtifact.project,
            sizeBytes: analyzedNodeArtifact.sizeBytes,
            ageDays: analyzedNodeArtifact.ageDays,
            recommendation: analyzedNodeArtifact.recommendation,
            selectedByDefault: true,
          },
        ],
        reclaimableSizeBytes: analyzedNodeArtifact.sizeBytes,
      },
      storageSummary: {
        status: 'available',
        totalBytes: 512 * 1024 ** 3,
        usedBytes: 318 * 1024 ** 3,
        availableBytes: 194 * 1024 ** 3,
        detectedDevelopmentBytes: analyzedArtifact.sizeBytes + analyzedNodeArtifact.sizeBytes,
        detectedSharePercent: 5.660377358490566,
        partial: false,
        warnings: [],
        recommendedBytes: analyzedNodeArtifact.sizeBytes,
        scopePath: '/workspace/dustfril',
        categories: ['Rust', 'Node'],
      },
    });
    vi.mocked(executeCleanup).mockResolvedValue({
      deletedPaths: [analyzedArtifact.path],
      failedPaths: [],
      freedSizeBytes: analyzedArtifact.sizeBytes,
    });

    render(<AppShell />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Analyze Workspace' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Analyze Workspace' }));
    await waitFor(() => expect(screen.getByRole('checkbox')).toBeChecked());

    fireEvent.click(screen.getByRole('button', { name: 'Cleanup' }));
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveTextContent('1 selected');
    expect(dialog).toHaveTextContent(analyzedArtifact.path);
    expect(dialog).not.toHaveTextContent(analyzedNodeArtifact.path);

    fireEvent.click(dialog.querySelector('.button-confirm') as HTMLButtonElement);
    await waitFor(() => expect(executeCleanup).toHaveBeenCalledOnce());
    expect(executeCleanup).toHaveBeenCalledWith(
      '/workspace',
      ['Rust', 'Node', 'Java'],
      'analysis-1',
      [{ path: analyzedArtifact.path, ecosystem: 'Rust' }],
      'Trash',
    );
  });

  it('refreshes volume capacity after permanent cleanup', async () => {
    const sizeBytes = 11 * 1024 ** 3;
    vi.mocked(analyzeWorkspace).mockResolvedValue({
      analysis: {
        artifacts: [{ ...analyzedArtifact, sizeBytes, recommendation: 'SafeToClean' }],
        totalSizeBytes: sizeBytes,
      },
      cleanupPlan: {
        analysisId: 'analysis-1',
        candidates: [
          {
            path: analyzedArtifact.path,
            ecosystem: analyzedArtifact.ecosystem,
            project: analyzedArtifact.project,
            sizeBytes,
            ageDays: analyzedArtifact.ageDays,
            recommendation: 'SafeToClean',
            selectedByDefault: true,
          },
        ],
        reclaimableSizeBytes: sizeBytes,
      },
      storageSummary: {
        status: 'available',
        totalBytes: 512 * 1024 ** 3,
        usedBytes: 318 * 1024 ** 3,
        availableBytes: 194 * 1024 ** 3,
        detectedDevelopmentBytes: sizeBytes,
        detectedSharePercent: 3.4591194968553455,
        partial: false,
        warnings: [],
        recommendedBytes: sizeBytes,
        scopePath: '/workspace/dustfril',
        categories: ['Rust'],
      },
    });
    vi.mocked(executeCleanup).mockResolvedValue({
      deletedPaths: [analyzedArtifact.path],
      failedPaths: [],
      freedSizeBytes: sizeBytes,
    });
    vi.mocked(refreshStorageVolume).mockResolvedValue({
      totalBytes: 512 * 1024 ** 3,
      usedBytes: 300 * 1024 ** 3,
      availableBytes: 212 * 1024 ** 3,
    });

    render(<AppShell />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Analyze Workspace' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Analyze Workspace' }));
    await waitFor(() => expect(screen.getByRole('checkbox')).toBeChecked());

    fireEvent.click(screen.getByRole('button', { name: 'Delete permanently' }));
    fireEvent.click(screen.getByRole('button', { name: 'Cleanup' }));
    fireEvent.click(screen.getByRole('dialog').querySelector('.button-confirm') as HTMLButtonElement);

    await waitFor(() => expect(refreshStorageVolume).toHaveBeenCalledWith('/workspace'));
    fireEvent.click(screen.getByRole('button', { name: 'Overview' }));
    expect(screen.getByText('300 GB used of 512 GB')).toBeInTheDocument();
  });
});
