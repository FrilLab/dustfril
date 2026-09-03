import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AppShell } from './AppShell';
import { analyzeWorkspace } from '../../lib/tauri';

vi.mock('../../lib/tauri', () => ({
  analyzeWorkspace: vi.fn(),
  chooseWorkspaceFolder: vi.fn(),
  defaultRoot: vi.fn().mockResolvedValue('/workspace'),
  executeCleanup: vi.fn(),
  loadActivityHistory: vi.fn().mockResolvedValue([]),
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
    await waitFor(() => expect(screen.getByText('Cleanup recommendations')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: 'Overview' }));
    expect(screen.queryByRole('button', { name: 'Review Cleanup' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^History/ }));
    expect(screen.queryByRole('button', { name: 'Review Cleanup' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Workspace/ }));
    expect(screen.getByRole('button', { name: 'Review Cleanup' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Overview' }));
    fireEvent.click(screen.getByRole('button', { name: 'Inspect dustfril target' }));

    expect(screen.getByRole('button', { name: /^Workspace/ })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('complementary', { name: 'Artifact inspector' })).toBeInTheDocument();
    expect(screen.getAllByText('/workspace/dustfril/target').length).toBeGreaterThan(0);
    expect(screen.getByRole('checkbox')).not.toBeChecked();
  });
});
