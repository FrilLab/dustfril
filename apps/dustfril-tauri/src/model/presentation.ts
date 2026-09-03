import type {
  ArtifactAnalysis,
  CleanupCandidate,
  Recommendation,
} from '../types/workflow';
import { formatAge, formatBytes, formatDate } from '../lib/format';

export type WorkspaceSummary = {
  artifactCount: number;
  reclaimableBytes: number;
  recommendedCount: number;
  reviewCount: number;
};

export function createWorkspaceSummary(
  artifacts: ArtifactAnalysis[] = [],
  reclaimableBytes = 0,
): WorkspaceSummary {
  return {
    artifactCount: artifacts.length,
    reclaimableBytes,
    recommendedCount: artifacts.filter((artifact) => artifact.recommendation === 'SafeToClean')
      .length,
    reviewCount: artifacts.filter((artifact) => artifact.recommendation === 'NeedsReview').length,
  };
}

export function filterArtifacts(artifacts: ArtifactAnalysis[], search: string) {
  const normalizedQuery = search.trim().toLowerCase();

  if (!normalizedQuery) {
    return artifacts;
  }

  return artifacts.filter((artifact) =>
    [
      artifact.project.displayName,
      artifact.project.root,
      leafName(artifact.path),
      artifact.path,
      artifact.ecosystem,
      artifact.recommendation,
      kindForArtifact(artifact.ecosystem),
    ].some((value) => value.toLowerCase().includes(normalizedQuery)),
  );
}

export function kindForArtifact(ecosystem: ArtifactAnalysis['ecosystem']) {
  return `${ecosystem} artifact`;
}

export function artifactLabel(artifact: ArtifactAnalysis) {
  return `${leafName(artifact.path)} · ${artifact.ecosystem}`;
}

export function recommendationLabel(recommendation: Recommendation) {
  switch (recommendation) {
    case 'SafeToClean':
      return 'Recommended';
    case 'NeedsReview':
      return 'Needs review';
    case 'Keep':
      return 'Keep';
  }
}

export function recommendationClass(recommendation: Recommendation) {
  switch (recommendation) {
    case 'SafeToClean':
      return 'recommendation recommendation-recommended';
    case 'NeedsReview':
      return 'recommendation recommendation-review';
    case 'Keep':
      return 'recommendation recommendation-keep';
  }
}

export function artifactDetailLines(
  artifact: ArtifactAnalysis,
  candidate: CleanupCandidate | undefined,
  selected: boolean,
) {
  return [
    ['Project', artifact.project.displayName],
    ['Project root', artifact.project.root],
    ['Artifact', leafName(artifact.path)],
    ['Path', artifact.path],
    ['Ecosystem', artifact.ecosystem],
    ['Kind', kindForArtifact(artifact.ecosystem)],
    ['Size', formatBytes(artifact.sizeBytes)],
    ['Modified', formatDate(artifact.lastModifiedMs)],
    ['Age', formatAge(artifact.ageDays)],
    ['Recommendation', recommendationLabel(artifact.recommendation)],
    ...(candidate
      ? [['Cleanup selection', selected ? 'Selected for review' : 'Not selected']]
      : []),
  ] as Array<[string, string]>;
}

export function leafName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : path;
}

export function pathBreadcrumb(path: string) {
  return path.split(/[\\/]/).filter(Boolean);
}
