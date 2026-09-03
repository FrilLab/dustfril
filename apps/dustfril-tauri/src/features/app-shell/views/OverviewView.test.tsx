import { fireEvent, render, screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { OverviewView } from './OverviewView';
import type { ActivityRecord, ArtifactAnalysis, CleanupCandidate } from '../../../types/workflow';

function artifact(
  project: string,
  name: string,
  sizeBytes: number,
  recommendation: ArtifactAnalysis['recommendation'],
): ArtifactAnalysis {
  return {
    path: `/workspace/${project}/${name}`,
    ecosystem: name === 'node_modules' ? 'Node' : 'Rust',
    project: {
      root: `/workspace/${project}`,
      displayName: project,
      ecosystem: name === 'node_modules' ? 'Node' : 'Rust',
    },
    sizeBytes,
    lastModifiedMs: Date.UTC(2026, 8, 1),
    ageDays: 30,
    recommendation,
  };
}

function candidate(item: ArtifactAnalysis): CleanupCandidate {
  return {
    path: item.path,
    ecosystem: item.ecosystem,
    project: item.project,
    sizeBytes: item.sizeBytes,
    ageDays: item.ageDays,
    recommendation: item.recommendation,
    selectedByDefault: item.recommendation === 'SafeToClean',
  };
}

const artifacts = [
  artifact('dustfril', 'target', 11 * 1024 ** 3, 'Keep'),
  artifact('frilvault', 'target', 3.6 * 1024 ** 3, 'NeedsReview'),
  artifact('portfolio', 'node_modules', 672 * 1024 ** 2, 'NeedsReview'),
  artifact('copc-adapter', 'target', 271 * 1024 ** 2, 'Keep'),
  artifact('viewer-web', 'node_modules', 200 * 1024 ** 2, 'SafeToClean'),
  artifact('small-project', 'target', 10 * 1024 ** 2, 'SafeToClean'),
];

const cleanupEntry: ActivityRecord = {
  id: 'cleanup-1',
  timestampMs: Date.UTC(2026, 8, 2, 11, 42),
  kind: 'Cleanup',
  result: {
    success: true,
    details: {
      target: '/workspace',
      mode: 'trash',
      freed: 1.8 * 1024 ** 3,
      items: [{ path: '/workspace/dustfril/target', project: 'dustfril', status: 'succeeded' }],
    },
  },
};

const newerScan: ActivityRecord = {
  id: 'scan-2',
  timestampMs: Date.UTC(2026, 8, 3),
  kind: 'Scan',
  result: { success: true, details: { path: '/workspace', artifacts: 6, size: 16 * 1024 ** 3 } },
};

function renderOverview(overrides: Partial<ComponentProps<typeof OverviewView>> = {}) {
  return render(
    <OverviewView
      root="/workspace/dustfril"
      analysisReady
      artifacts={artifacts}
      candidates={artifacts.map(candidate)}
      reclaimableBytes={210 * 1024 ** 2}
      historyEntries={[cleanupEntry, newerScan]}
      error={null}
      onInspectArtifact={vi.fn()}
      onOpenHistory={vi.fn()}
      {...overrides}
    />,
  );
}

describe('OverviewView', () => {
  it('projects recommendation state into compact actionable summaries', () => {
    renderOverview();

    expect(screen.getByText('Reclaimable now')).toBeInTheDocument();
    expect(screen.getByText('210 MB')).toBeInTheDocument();
    expect(screen.getAllByText('2 artifacts')).toHaveLength(2);
    expect(screen.getByText('4.3 GB')).toBeInTheDocument();
    expect(screen.getAllByText('2 artifacts', { selector: '.overview-summary-card > span' })).toHaveLength(2);
    expect(screen.queryByText('Discovered ecosystems')).not.toBeInTheDocument();
    expect(screen.queryByText('Review recommendations based on inactivity age')).not.toBeInTheDocument();
  });

  it('shows the five largest artifacts and navigates with the stable path identity', () => {
    const onInspectArtifact = vi.fn();
    renderOverview({ onInspectArtifact });

    expect(screen.getByText('Largest artifacts')).toBeInTheDocument();
    expect(screen.getByText('11 GB')).toBeInTheDocument();
    expect(screen.getByText('3.6 GB')).toBeInTheDocument();
    expect(screen.queryByText('10 MB')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Inspect dustfril target' }));
    expect(onInspectArtifact).toHaveBeenCalledWith('/workspace/dustfril/target');
  });

  it('uses the latest cleanup rather than a newer scan and offers History entry navigation', () => {
    const onOpenHistory = vi.fn();
    renderOverview({ onOpenHistory });

    expect(screen.getByText(/Move to Trash · Success/)).toBeInTheDocument();
    expect(screen.getByText(/1 artifact · 1.8 GB/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'View history' }));
    expect(onOpenHistory).toHaveBeenCalledOnce();
  });

  it('does not present zeroes as analysis when the workspace has not been scanned', () => {
    renderOverview({
      analysisReady: false,
      artifacts: [],
      candidates: [],
      reclaimableBytes: 0,
      historyEntries: [],
    });

    expect(screen.getAllByText('Not analyzed')).toHaveLength(2);
    expect(screen.getByText('Analyze this workspace to see cleanup insights.')).toBeInTheDocument();
    expect(screen.queryByText('0 B')).not.toBeInTheDocument();
    expect(screen.getByText('No cleanup operations yet.')).toBeInTheDocument();
  });
});
