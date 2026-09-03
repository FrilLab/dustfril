import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { WorkspaceView } from './WorkspaceView';
import type { ArtifactAnalysis } from '../../../types/workflow';

const artifact: ArtifactAnalysis = {
  path: '/workspace/dustfril/target',
  ecosystem: 'Rust',
  project: {
    root: '/workspace/dustfril',
    displayName: 'dustfril',
    ecosystem: 'Rust',
  },
  sizeBytes: 1024,
  lastModifiedMs: null,
  ageDays: 30,
  recommendation: 'Keep',
};

function renderWorkspace(deleteMode: 'Trash' | 'Permanent' = 'Trash') {
  return render(
    <WorkspaceView
      artifacts={[artifact]}
      artifactCount={1}
      candidates={[]}
      reclaimableBytes={0}
      selectedItemId={artifact.path}
      selectedPaths={[]}
      selectedCandidateBytes={0}
      canReviewCleanup={false}
      deleteMode={deleteMode}
      deleteModes={['Trash', 'Permanent']}
      cleanupAgeDays={30}
      lastAnalysisAtMs={Date.UTC(2026, 8, 3)}
      busy={false}
      analysisReady
      error={null}
      onSelectItem={vi.fn()}
      onCloseInspector={vi.fn()}
      onTogglePath={vi.fn()}
      onDeleteModeChange={vi.fn()}
      onCleanupAgeChange={vi.fn()}
      onRequestCleanup={vi.fn()}
    />,
  );
}

describe('WorkspaceView information hierarchy', () => {
  it('keeps policy and cleanup controls without permanent recommendation or Trash copy', () => {
    renderWorkspace();

    expect(screen.getByText('Cleanup recommendations')).toBeInTheDocument();
    expect(screen.getByLabelText('Cleanup age')).toHaveValue('30');
    expect(screen.getByRole('columnheader', { name: 'Project' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Artifact' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Size' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Modified' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Size' })).toHaveAttribute('aria-sort', 'descending');
    expect(screen.getByRole('button', { name: 'Move to Trash' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete permanently' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Review Cleanup' })).toBeDisabled();
    expect(screen.getByText('0 selected · 0 B')).toBeInTheDocument();
    expect(screen.queryByText(/Review recommendations based on inactivity age/)).not.toBeInTheDocument();
    expect(screen.queryByText('Trash is the default cleanup mode.')).not.toBeInTheDocument();
    expect(screen.queryByText(/cannot be undone/)).not.toBeInTheDocument();
  });

  it('keeps the existing artifact inspector available on demand', () => {
    renderWorkspace();

    expect(screen.getByRole('complementary', { name: 'Artifact inspector' })).toBeInTheDocument();
    expect(screen.getAllByText('/workspace/dustfril/target').length).toBeGreaterThan(0);
  });

  it('toggles the active sort direction and exposes only the active indicator', () => {
    renderWorkspace();

    fireEvent.click(screen.getByRole('button', { name: 'Project' }));
    expect(screen.getByRole('columnheader', { name: 'Project' })).toHaveAttribute('aria-sort', 'ascending');
    expect(screen.getByRole('columnheader', { name: 'Size' })).toHaveAttribute('aria-sort', 'none');

    fireEvent.click(screen.getByRole('button', { name: 'Project' }));
    expect(screen.getByRole('columnheader', { name: 'Project' })).toHaveAttribute('aria-sort', 'descending');
  });
});
