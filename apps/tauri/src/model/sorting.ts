import type { ActivityRecord, ArtifactAnalysis, Recommendation } from '../types/workflow';
import {
  historyActionSortKey,
  historyResultSortKey,
  historyStatusRank,
  historyTargetLabel,
} from './activity';
import { leafName } from './presentation';

export type SortDirection = 'asc' | 'desc';

export type WorkspaceSortColumn = 'project' | 'artifact' | 'size' | 'modified' | 'status';
export type HistorySortColumn = 'time' | 'action' | 'target' | 'result' | 'status';

export type WorkspaceSortState = {
  column: WorkspaceSortColumn;
  direction: SortDirection;
};

export type HistorySortState = {
  column: HistorySortColumn;
  direction: SortDirection;
};

export function sortArtifacts(
  artifacts: ArtifactAnalysis[],
  sort: WorkspaceSortState,
): ArtifactAnalysis[] {
  return artifacts
    .map((artifact, index) => ({ artifact, index }))
    .sort((left, right) => {
      const comparison = compareArtifacts(left.artifact, right.artifact, sort.column);
      return comparison
        ? sort.direction === 'asc'
          ? comparison
          : -comparison
        : left.index - right.index;
    })
    .map(({ artifact }) => artifact);
}

export function sortActivityRecords(
  entries: ActivityRecord[],
  sort: HistorySortState,
): ActivityRecord[] {
  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((left, right) => {
      const comparison = compareActivity(left.entry, right.entry, sort.column);
      if (comparison) {
        return sort.direction === 'asc' ? comparison : -comparison;
      }

      return compareText(left.entry.id, right.entry.id) || left.index - right.index;
    })
    .map(({ entry }) => entry);
}

function compareArtifacts(
  left: ArtifactAnalysis,
  right: ArtifactAnalysis,
  column: WorkspaceSortColumn,
) {
  switch (column) {
    case 'project':
      return (
        compareText(left.project.displayName, right.project.displayName) ||
        compareText(left.project.root, right.project.root) ||
        compareText(leafName(left.path), leafName(right.path)) ||
        compareText(left.path, right.path)
      );
    case 'artifact':
      return (
        compareText(leafName(left.path), leafName(right.path)) ||
        compareText(left.ecosystem, right.ecosystem) ||
        compareText(left.project.displayName, right.project.displayName) ||
        compareText(left.path, right.path)
      );
    case 'size':
      return (
        compareNumber(left.sizeBytes, right.sizeBytes) ||
        compareText(left.project.displayName, right.project.displayName) ||
        compareText(left.path, right.path)
      );
    case 'modified':
      return (
        compareNullableNumber(left.lastModifiedMs, right.lastModifiedMs) ||
        compareText(left.project.displayName, right.project.displayName) ||
        compareText(left.path, right.path)
      );
    case 'status':
      return (
        compareNumber(recommendationRank(left.recommendation), recommendationRank(right.recommendation)) ||
        compareText(left.project.displayName, right.project.displayName) ||
        compareText(left.path, right.path)
      );
  }
}

function compareActivity(left: ActivityRecord, right: ActivityRecord, column: HistorySortColumn) {
  switch (column) {
    case 'time':
      return compareNumber(left.timestampMs, right.timestampMs);
    case 'action':
      return (
        compareText(historyActionSortKey(left.kind), historyActionSortKey(right.kind)) ||
        compareText(historyTargetLabel(left), historyTargetLabel(right)) ||
        compareNumber(left.timestampMs, right.timestampMs)
      );
    case 'target':
      return (
        compareText(historyTargetLabel(left), historyTargetLabel(right)) ||
        compareText(left.kind, right.kind) ||
        compareNumber(left.timestampMs, right.timestampMs)
      );
    case 'result': {
      const leftKey = historyResultSortKey(left);
      const rightKey = historyResultSortKey(right);
      return (
        compareNumber(leftKey.bytes, rightKey.bytes) ||
        compareNumber(leftKey.itemCount, rightKey.itemCount) ||
        compareText(leftKey.fallbackText, rightKey.fallbackText) ||
        compareNumber(left.timestampMs, right.timestampMs)
      );
    }
    case 'status':
      return (
        compareNumber(historyStatusRank(left), historyStatusRank(right)) ||
        compareText(historyTargetLabel(left), historyTargetLabel(right)) ||
        compareNumber(left.timestampMs, right.timestampMs)
      );
  }
}

function recommendationRank(recommendation: Recommendation) {
  switch (recommendation) {
    case 'Keep':
      return 1;
    case 'NeedsReview':
      return 2;
    case 'SafeToClean':
      return 3;
  }
}

function compareNumber(left: number, right: number) {
  return left === right ? 0 : left < right ? -1 : 1;
}

function compareNullableNumber(left: number | null, right: number | null) {
  if (left === null && right === null) {
    return 0;
  }
  if (left === null) {
    return 1;
  }
  if (right === null) {
    return -1;
  }

  return compareNumber(left, right);
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
