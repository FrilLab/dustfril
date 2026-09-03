import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HistoryList } from './HistoryList';
import type { ActivityRecord } from '../../types/workflow';

const scanEntry: ActivityRecord = {
  id: 'scan-1',
  timestampMs: Date.UTC(2026, 8, 3, 4, 5),
  kind: 'Scan',
  result: {
    success: true,
    details: {
      path: '/Users/mars112/code/dustfril',
      artifacts: 2,
      size: 11 * 1024 ** 3,
      accessSummary: {
        root: '/Users/mars112/code/dustfril',
        directoriesVisited: 363,
        filesInspected: 7,
        metadataFilesInspected: 7,
        artifactCandidates: 2,
        symlinksSkipped: 0,
        failures: 0,
        failureSamples: [],
      },
    },
  },
};

const cleanupEntry: ActivityRecord = {
  id: 'cleanup-1',
  timestampMs: Date.UTC(2026, 8, 3, 4, 10),
  kind: 'Cleanup',
  result: {
    success: false,
    details: {
      target: '/Users/mars112/code',
      mode: 'trash',
      freed: 2.3 * 1024 ** 3,
      items: [
        {
          path: '/Users/mars112/code/dustfril/target',
          project: 'dustfril',
          status: 'succeeded',
          size: 2.3 * 1024 ** 3,
        },
        {
          path: '/Users/mars112/code/project/node_modules',
          project: 'project',
          status: 'failed',
          reason: 'NotFound',
        },
      ],
      deleted: ['/Users/mars112/code/dustfril/target'],
      failed: [{ path: '/Users/mars112/code/project/node_modules', reason: 'NotFound' }],
    },
  },
};

const permanentCleanupEntry: ActivityRecord = {
  ...cleanupEntry,
  id: 'cleanup-2',
  result: {
    ...cleanupEntry.result,
    success: true,
    details: {
      ...cleanupEntry.result.details,
      mode: 'permanent',
      items: cleanupEntry.result.details.items?.slice(0, 1) ?? [],
      failed: [],
    },
  },
};

const failedCleanupEntry: ActivityRecord = {
  ...cleanupEntry,
  id: 'cleanup-failed',
  result: {
    success: false,
    details: {
      ...cleanupEntry.result.details,
      items: cleanupEntry.result.details.items?.slice(1) ?? [],
      deleted: [],
      failed: [{ path: '/Users/mars112/code/project/node_modules', reason: 'NotFound' }],
    },
  },
};

const securityEntry: ActivityRecord = {
  id: 'security-1',
  timestampMs: Date.UTC(2026, 8, 3, 4, 20),
  kind: 'Security',
  result: {
    success: true,
    details: {
      path: '/Users/mars112/code/dustfril',
      ecosystems: [],
      findingCount: 0,
      highestRisk: 'None',
      findings: [],
    },
  },
};

describe('HistoryList', () => {
  it('renders compact operation rows and keeps diagnostics in the drawer', () => {
    render(<HistoryList entries={[scanEntry]} />);

    const expectedTime = new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    }).format(new Date(scanEntry.timestampMs));
    expect(screen.getByText(expectedTime)).toBeInTheDocument();
    expect(screen.getByText('Scan')).toBeInTheDocument();
    expect(screen.getByText('dustfril')).toBeInTheDocument();
    expect(screen.getByText(/2 artifacts · 11 GB/)).toBeInTheDocument();
    expect(screen.getByText('Success')).toBeInTheDocument();
    expect(screen.queryByText('Directories')).not.toBeInTheDocument();

    const row = screen.getByRole('row', { name: /Inspect Scan activity/ });
    expect(row).toHaveAttribute('tabindex', '0');
    fireEvent.keyDown(row, { key: 'Enter' });

    expect(screen.getByRole('heading', { name: 'Scan details' })).toBeInTheDocument();
    expect(screen.getByText('/Users/mars112/code/dustfril')).toBeInTheDocument();
    expect(screen.getByText('Directories')).toBeInTheDocument();
    expect(screen.getByText('363')).toBeInTheDocument();
    expect(screen.getByText('Files inspected')).toBeInTheDocument();
    expect(screen.getByText('Metadata files')).toBeInTheDocument();
    expect(screen.getByText('Artifact candidates')).toBeInTheDocument();
    expect(screen.getByText('Symlinks skipped')).toBeInTheDocument();
    expect(screen.getByText('Failures')).toBeInTheDocument();
  });

  it('updates the drawer for another row, exposes cleanup detail, and closes cleanly', () => {
    render(<HistoryList entries={[scanEntry, cleanupEntry]} />);

    expect(screen.getByText('Partial failure')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('row', { name: /Inspect Scan activity/ }));
    fireEvent.click(screen.getByRole('row', { name: /Inspect Cleanup activity/ }));

    expect(screen.getByRole('heading', { name: 'Cleanup details' })).toBeInTheDocument();
    expect(screen.getByText('Move to Trash')).toBeInTheDocument();
    expect(screen.getByText('Affected targets')).toBeInTheDocument();
    expect(screen.getByText('/Users/mars112/code/dustfril/target')).toBeInTheDocument();
    expect(screen.queryByText('Scan access')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Close history details' }));
    expect(screen.queryByRole('heading', { name: 'Cleanup details' })).not.toBeInTheDocument();
    expect(screen.getByText('Partial failure')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('row', { name: /Inspect Scan activity/ }));
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('heading', { name: 'Scan details' })).not.toBeInTheDocument();
  });

  it('labels permanent cleanup details distinctly from Trash cleanup', () => {
    render(<HistoryList entries={[permanentCleanupEntry]} />);

    fireEvent.click(screen.getByRole('row', { name: /Inspect Cleanup activity/ }));

    expect(screen.getByText('Delete permanently')).toBeInTheDocument();
    expect(screen.queryByText('Move to Trash')).not.toBeInTheDocument();
    expect(screen.getByText('Deleted permanently')).toBeInTheDocument();
  });

  it('keeps all-failed cleanup operations distinct from partial failures', () => {
    render(<HistoryList entries={[failedCleanupEntry]} />);

    expect(screen.getByText('Failed')).toBeInTheDocument();
    expect(screen.queryByText('Partial failure')).not.toBeInTheDocument();
  });

  it('preserves the recorded empty security ecosystem scope', () => {
    render(<HistoryList entries={[securityEntry]} />);

    fireEvent.click(screen.getByRole('row', { name: /Inspect Security activity/ }));

    expect(screen.getByText('Ecosystems').parentElement).toHaveTextContent('None');
    expect(screen.queryByText('All')).not.toBeInTheDocument();
  });
});
