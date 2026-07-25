import type { ArtifactAnalysis, CleanupCandidate, Recommendation } from '../types/workflow';
import { formatAge, formatBytes, formatDate } from '../lib/format';
import type { BrowserItem, WorkspaceSummary } from './types';

export function createWorkspaceSummary(artifacts: ArtifactAnalysis[] = []): WorkspaceSummary {
  return artifacts.reduce(
    (accumulator, artifact) => {
      if (artifact.recommendation === 'Keep') {
        accumulator.keepCount += 1;
      }
      if (artifact.recommendation === 'NeedsReview') {
        accumulator.reviewCount += 1;
        accumulator.reviewBytes += artifact.sizeBytes;
      }
      if (artifact.recommendation === 'SafeToClean') {
        accumulator.safeCount += 1;
        accumulator.safeBytes += artifact.sizeBytes;
      }
      return accumulator;
    },
    {
      keepCount: 0,
      reviewCount: 0,
      safeCount: 0,
      reviewBytes: 0,
      safeBytes: 0,
    },
  );
}

export function createAnalysisItems(artifacts: ArtifactAnalysis[] = []): BrowserItem[] {
  return artifacts.map((artifact) => ({
    id: `analysis:${artifact.path}`,
    title: leafName(artifact.path),
    subtitle: artifact.path,
    meta: `${formatBytes(artifact.sizeBytes)} · ${formatAge(artifact.ageDays)}`,
    badge: artifact.recommendation,
    accent: recommendationAccent(artifact.recommendation),
    kind:
      artifact.recommendation === 'SafeToClean'
        ? 'safe'
        : artifact.recommendation === 'NeedsReview'
          ? 'warning'
          : 'folder',
    path: artifact.path,
    detailLines: [
      `Ecosystem: ${artifact.ecosystem}`,
      `Recommendation: ${artifact.recommendation}`,
      `Size: ${formatBytes(artifact.sizeBytes)}`,
      `Modified: ${formatDate(artifact.lastModifiedMs)}`,
      `Age: ${formatAge(artifact.ageDays)}`,
    ],
  }));
}

export function createCleanupItems(
  candidates: CleanupCandidate[] = [],
  selectedPaths: string[],
  deleteMode: string,
): BrowserItem[] {
  return candidates.map((candidate) => {
    const selected = selectedPaths.includes(candidate.path);

    return {
      id: `cleanup:${candidate.path}`,
      title: leafName(candidate.path),
      subtitle: candidate.path,
      meta: `${formatBytes(candidate.sizeBytes)} · ${formatAge(candidate.ageDays)}`,
      badge: selected ? 'Selected' : 'Queued',
      accent: selected
        ? 'border-cyan-300/40 bg-cyan-400/14 text-cyan-50'
        : 'border-white/10 bg-white/6 text-slate-200',
      kind: selected ? 'safe' : 'folder',
      path: candidate.path,
      detailLines: [
        `Ecosystem: ${candidate.ecosystem}`,
        `Stage: ${selected ? 'Included in execution' : 'Not selected'}`,
        `Size: ${formatBytes(candidate.sizeBytes)}`,
        `Age: ${formatAge(candidate.ageDays)}`,
        `Delete mode: ${deleteMode}`,
      ],
    };
  });
}

export function createScanItems(
  artifacts: Array<{ path: string; ecosystem: string }> = [],
): BrowserItem[] {
  return artifacts.map((artifact) => ({
    id: `scan:${artifact.path}`,
    title: leafName(artifact.path),
    subtitle: artifact.path,
    meta: artifact.ecosystem,
    badge: 'Scanned',
    accent: 'border-amber-300/30 bg-amber-400/10 text-amber-50',
    kind: 'folder',
    path: artifact.path,
    detailLines: [`Ecosystem: ${artifact.ecosystem}`, `Path: ${artifact.path}`],
  }));
}

export function filterItems(items: BrowserItem[], search: string) {
  const normalizedQuery = search.trim().toLowerCase();

  if (!normalizedQuery) {
    return items;
  }

  return items.filter((item) =>
    [item.title, item.subtitle, item.badge, item.meta].some((value) =>
      value.toLowerCase().includes(normalizedQuery),
    ),
  );
}

function recommendationAccent(recommendation: Recommendation) {
  if (recommendation === 'Keep') {
    return 'border-emerald-400/25 bg-emerald-400/10 text-emerald-100';
  }
  if (recommendation === 'NeedsReview') {
    return 'border-amber-400/25 bg-amber-400/10 text-amber-100';
  }
  return 'border-cyan-400/25 bg-cyan-400/10 text-cyan-100';
}

function leafName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : path;
}
