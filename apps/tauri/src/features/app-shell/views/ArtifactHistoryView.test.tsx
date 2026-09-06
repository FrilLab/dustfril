import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ArtifactHistoryView } from './ArtifactHistoryView';
import type { ActivityRecord, ArtifactSnapshotHistory } from '../../../types/workflow';

const scanEntry: ActivityRecord = {
  id: 'scan-2',
  timestampMs: Date.UTC(2026, 8, 3, 4, 5),
  kind: 'Scan',
  result: {
    success: true,
    details: {
      path: '/workspace',
      accessSummary: {
        root: '/workspace',
        directoriesVisited: 12,
        filesInspected: 9,
        metadataFilesInspected: 7,
        artifactCandidates: 3,
        symlinksSkipped: 2,
        failures: 1,
        failureSamples: [{ path: 'restricted', reason: 'permission denied' }],
      },
    },
  },
};

const history: ArtifactSnapshotHistory = {
  entries: [
    {
      status: 'baselineCreated',
      snapshot: {
        workspaceId: '/workspace',
        timestamp: '2026-09-01T00:00:00Z',
        artifacts: [],
      },
      previousSnapshot: null,
      changes: [],
    },
    {
      status: 'compared',
      snapshot: {
        workspaceId: '/workspace',
        timestamp: '2026-09-03T00:00:00Z',
        artifacts: [],
      },
      previousSnapshot: {
        workspaceId: '/workspace',
        timestamp: '2026-09-01T00:00:00Z',
        artifacts: [],
      },
      changes: [
        { path: 'target', ecosystem: 'Rust', kind: 'new', previousSizeBytes: null, currentSizeBytes: 2048, deltaBytes: 2048 },
        { path: 'node_modules', ecosystem: 'Node', kind: 'removed', previousSizeBytes: 4096, currentSizeBytes: null, deltaBytes: -4096 },
        { path: 'build', ecosystem: 'Java', kind: 'sizeIncreased', previousSizeBytes: 1024, currentSizeBytes: 3072, deltaBytes: 2048 },
        { path: 'other/target', ecosystem: 'Rust', kind: 'sizeDecreased', previousSizeBytes: 3072, currentSizeBytes: 1024, deltaBytes: -2048 },
        { path: 'other/node_modules', ecosystem: 'Node', kind: 'unchanged', previousSizeBytes: 512, currentSizeBytes: 512, deltaBytes: 0 },
      ],
    },
  ],
  retainedSnapshotCount: 2,
  retentionLimit: 32,
};

const props = {
  root: '/workspace',
  status: 'success' as const,
  error: null,
  persistenceWarning: null,
};

describe('ArtifactHistoryView', () => {
  it('distinguishes a workspace with no scan history', () => {
    render(<ArtifactHistoryView {...props} history={{ entries: [], retainedSnapshotCount: 0, retentionLimit: 32 }} scanEntry={null} />);

    expect(screen.getByText(/No scan has been run for this workspace yet/)).toBeInTheDocument();
    expect(screen.getByText(/no generated-artifact baseline to compare/)).toBeInTheDocument();
  });

  it('renders the bounded summary, baseline, all Core change states, and exact signed deltas', () => {
    render(<ArtifactHistoryView {...props} history={history} scanEntry={scanEntry} />);

    expect(screen.getByText('Scan access summary')).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
    expect(screen.getByText('Representative failure samples')).toBeInTheDocument();
    expect(screen.getByText('Baseline created')).toBeInTheDocument();
    expect(screen.getByText('New')).toBeInTheDocument();
    expect(screen.getByText('Removed')).toBeInTheDocument();
    expect(screen.getByText('Size increased')).toBeInTheDocument();
    expect(screen.getByText('Size decreased')).toBeInTheDocument();
    expect(screen.getByText('Unchanged')).toBeInTheDocument();
    expect(screen.getAllByText('+2,048 bytes')).not.toHaveLength(0);
    expect(screen.getByText('-4,096 bytes')).toBeInTheDocument();
    expect(screen.getByText('0 bytes')).toBeInTheDocument();
    expect(screen.getByText(/at most 32 snapshots/)).toBeInTheDocument();
  });

  it('keeps persistence warnings visible while displaying retained history', () => {
    render(
      <ArtifactHistoryView
        {...props}
        history={history}
        scanEntry={scanEntry}
        persistenceWarning="Failed to record artifact snapshot"
      />,
    );

    expect(screen.getByRole('status')).toHaveTextContent('Failed to record artifact snapshot');
    expect(screen.getByText('Generated artifact snapshots')).toBeInTheDocument();
  });

  it('explains when the oldest retained entry has no available predecessor', () => {
    render(
      <ArtifactHistoryView
        {...props}
        history={{
          entries: [
            {
              status: 'comparisonUnavailable',
              snapshot: {
                workspaceId: '/workspace',
                timestamp: '2026-09-01T00:00:00Z',
                artifacts: [],
              },
              previousSnapshot: null,
              changes: [],
            },
          ],
          retainedSnapshotCount: 32,
          retentionLimit: 32,
        }}
        scanEntry={scanEntry}
      />,
    );

    expect(screen.getByText('Comparison unavailable')).toBeInTheDocument();
    expect(screen.getByText(/retention limit was reached/)).toBeInTheDocument();
  });

  it('does not call a missing activity record proof that no scan ever ran', () => {
    render(<ArtifactHistoryView {...props} history={history} scanEntry={null} />);

    expect(screen.getByText(/No retained scan activity is available/)).toBeInTheDocument();
    expect(screen.queryByText(/No scan has been run for this workspace yet/)).not.toBeInTheDocument();
    expect(screen.getByText('Compared')).toBeInTheDocument();
  });

  it('shows malformed or unsupported persisted state as an error', () => {
    render(
      <ArtifactHistoryView
        root="/workspace"
        history={null}
        status="error"
        error="unsupported artifact snapshot state version: 2"
        scanEntry={null}
        persistenceWarning={null}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Artifact history unavailable' })).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('unsupported artifact snapshot state version: 2');
  });
});
