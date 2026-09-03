import { formatBytes } from '../lib/format';
import type {
  ActivityDetails,
  ActivityKind,
  ActivityRecord,
  CleanupActivityItem,
} from '../types/workflow';
import { leafName } from './presentation';

export type HistoryResultSortKey = {
  bytes: number;
  itemCount: number;
  fallbackText: string;
};

export function cleanupItems(details: ActivityDetails): CleanupActivityItem[] {
  if (details.items?.length) {
    return details.items;
  }

  return [
    ...(details.deleted ?? []).map((path) => ({ path, status: 'succeeded' as const })),
    ...(details.failed ?? []).map((failure) => ({
      path: failure.path,
      status: 'failed' as const,
      reason: failure.reason,
    })),
  ];
}

export function cleanupItemCount(details: ActivityDetails) {
  return cleanupItems(details).length;
}

export function cleanupModeLabel(mode: ActivityDetails['mode']) {
  return mode === 'trash'
    ? 'Move to Trash'
    : mode === 'permanent'
      ? 'Delete permanently'
      : 'Unknown';
}

export function historyStatusLabel(entry: ActivityRecord) {
  const failureCount = historyFailureCount(entry);
  if (failureCount > 0 && (entry.kind !== 'Cleanup' || cleanupSuccessCount(entry) > 0)) {
    return 'Partial failure';
  }

  return entry.result.success ? 'Success' : 'Failed';
}

export function historyStatusRank(entry: ActivityRecord) {
  switch (historyStatusLabel(entry)) {
    case 'Failed':
      return 3;
    case 'Partial failure':
      return 2;
    case 'Success':
      return 1;
  }
}

export function historyTargetLabel(entry: ActivityRecord) {
  const details = entry.result.details;

  if (entry.kind === 'Cleanup') {
    const projects = Array.from(
      new Set(
        (details.items ?? [])
          .map((item) => item.project)
          .filter((project): project is string => Boolean(project)),
      ),
    );
    projects.sort(compareText);
    if (projects.length === 1) {
      return projects[0];
    }
    if (projects.length > 1) {
      return `${projects[0]} + ${projects.length - 1}`;
    }
  }

  return conciseTargetName(details.target ?? details.path ?? firstCleanupPath(details));
}

export function historyResultLabel(entry: ActivityRecord) {
  const details = entry.result.details;

  switch (entry.kind) {
    case 'Scan': {
      const count = details.artifacts ?? 0;
      const failures = details.accessSummary?.failures ?? 0;
      return `${count} artifact${count === 1 ? '' : 's'} · ${formatBytes(details.size ?? 0)}${
        failures ? ` · ${failures} failure${failures === 1 ? '' : 's'}` : ''
      }`;
    }
    case 'Cleanup': {
      const items = cleanupItems(details);
      const succeeded = items.filter((item) => item.status === 'succeeded').length;
      const failed = items.filter((item) => item.status === 'failed').length;
      const verb = details.mode === 'trash' ? 'moved to Trash' : 'deleted';
      const result = `${succeeded} item${succeeded === 1 ? '' : 's'} · ${formatBytes(
        details.freed ?? 0,
      )} ${verb}`;
      return failed ? `${result} · ${failed} failed` : result;
    }
    case 'Security': {
      const count = details.findingCount ?? details.findings?.length ?? 0;
      return `${count} finding${count === 1 ? '' : 's'} · ${details.highestRisk ?? 'None'} risk`;
    }
  }
}

/**
 * Numeric result data kept separate from the compact result label. This lets
 * History sort heterogeneous operations without reverse-engineering UI text.
 */
export function historyResultSortKey(entry: ActivityRecord): HistoryResultSortKey {
  const details = entry.result.details;

  switch (entry.kind) {
    case 'Scan':
      return {
        bytes: details.size ?? 0,
        itemCount: details.artifacts ?? 0,
        fallbackText: details.path ?? '',
      };
    case 'Cleanup': {
      const items = cleanupItems(details);
      return {
        bytes: details.freed ?? 0,
        itemCount: items.filter((item) => item.status === 'succeeded').length,
        fallbackText: details.mode ?? '',
      };
    }
    case 'Security':
      return {
        bytes: 0,
        itemCount: details.findingCount ?? details.findings?.length ?? 0,
        fallbackText: details.highestRisk ?? '',
      };
  }
}

export function historyActionSortKey(kind: ActivityKind) {
  return kind;
}

function historyFailureCount(entry: ActivityRecord) {
  if (entry.kind === 'Scan') {
    return entry.result.details.accessSummary?.failures ?? 0;
  }

  if (entry.kind === 'Cleanup') {
    const recordedFailures = entry.result.details.failed?.length ?? 0;
    const contextualFailures =
      entry.result.details.items?.filter((item) => item.status === 'failed').length ?? 0;
    return Math.max(recordedFailures, contextualFailures);
  }

  return 0;
}

function cleanupSuccessCount(entry: ActivityRecord) {
  return entry.kind === 'Cleanup'
    ? cleanupItems(entry.result.details).filter((item) => item.status === 'succeeded').length
    : 0;
}

function firstCleanupPath(details: ActivityDetails) {
  return details.deleted?.[0] ?? details.failed?.[0]?.path ?? 'Unknown';
}

function conciseTargetName(path: string) {
  return path === 'Unknown' ? path : leafName(path);
}

function compareText(left: string, right: string) {
  const leftLower = left.toLowerCase();
  const rightLower = right.toLowerCase();
  return leftLower === rightLower
    ? left === right
      ? 0
      : left < right
        ? -1
        : 1
    : leftLower < rightLower
      ? -1
      : 1;
}
