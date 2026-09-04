import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AppShell } from './AppShell';
import { analyzeWorkspace, clearActivityHistory, loadActivityHistory } from '../../lib/tauri';

vi.mock('../../lib/tauri', () => ({
  analyzeWorkspace: vi.fn(),
  chooseWorkspaceFolder: vi.fn(),
  defaultRoot: vi.fn().mockResolvedValue('/workspace'),
  executeCleanup: vi.fn(),
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
});
