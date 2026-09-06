import type {
  ActivityRecord,
  ArtifactChangeKind,
  ArtifactSnapshotStatus,
} from '../types/workflow';

export function latestScanForWorkspace(entries: ActivityRecord[], root: string) {
  const normalizedRoot = normalizePath(root);

  return (
    entries
      .filter((entry) => {
        if (entry.kind !== 'Scan') {
          return false;
        }

        const details = entry.result.details;
        const recordedRoot = details.path ?? details.accessSummary?.root;
        return recordedRoot ? normalizePath(recordedRoot) === normalizedRoot : false;
      })
      .sort((left, right) => right.timestampMs - left.timestampMs)[0] ?? null
  );
}

export function scanExecutionLabel(entry: ActivityRecord) {
  if (!entry.result.success) {
    return 'Failed';
  }

  return entry.result.details.accessSummary?.failures
    ? 'Completed with warnings'
    : 'Completed';
}

export function changeKindLabel(kind: ArtifactChangeKind) {
  switch (kind) {
    case 'new':
      return 'New';
    case 'removed':
      return 'Removed';
    case 'sizeIncreased':
      return 'Size increased';
    case 'sizeDecreased':
      return 'Size decreased';
    case 'unchanged':
      return 'Unchanged';
  }
}

export function snapshotStatusLabel(status: ArtifactSnapshotStatus) {
  switch (status) {
    case 'baselineCreated':
      return 'Baseline created';
    case 'compared':
      return 'Compared';
    case 'comparisonUnavailable':
      return 'Comparison unavailable';
  }
}

function normalizePath(path: string) {
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '');
  return normalized || '/';
}
