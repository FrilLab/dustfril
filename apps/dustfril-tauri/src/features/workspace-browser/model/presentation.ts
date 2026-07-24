import type {
  ArtifactAnalysis,
  CleanupCandidate,
  LifecycleScript,
  Recommendation,
  RiskLevel,
} from '../../../types/workflow';
import { formatAge, formatBytes, formatCount, formatDate } from '../../../lib/format';
import type {
  BrowserItem,
  BrowserPane,
  PaneConfig,
  StatusMetric,
  TotalsMetric,
  WorkspaceSummary,
} from './types';

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

export function createPaneConfigs(args: {
  analysisCount: number;
  cleanupCount: number;
  scanCount: number;
  auditCount: number;
}): PaneConfig[] {
  return [
    {
      key: 'analysis',
      title: 'Artifact Library',
      description: 'Age, size, and recommendation applied to every artifact.',
      count: args.analysisCount,
      accent: 'text-slate-100',
    },
    {
      key: 'cleanup',
      title: 'Cleanup Queue',
      description: 'Candidates staged for deletion execution.',
      count: args.cleanupCount,
      accent: 'text-cyan-200',
    },
    {
      key: 'scan',
      title: 'Scan Index',
      description: 'Raw artifact paths discovered from the workspace scan.',
      count: args.scanCount,
      accent: 'text-amber-100',
    },
    {
      key: 'audit',
      title: 'Script Audit',
      description: 'Lifecycle scripts with risk grading.',
      count: args.auditCount,
      accent: 'text-rose-100',
    },
  ];
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

export function createAuditItems(auditScripts: LifecycleScript[] = []): BrowserItem[] {
  return auditScripts.map((script) => ({
    id: `audit:${script.package}:${script.scriptType}:${script.command}`,
    title: `${script.package} · ${script.scriptType}`,
    subtitle: script.command,
    meta: `Risk ${script.riskLevel}`,
    badge: script.riskLevel,
    accent: riskAccent(script.riskLevel),
    kind: script.riskLevel === 'High' ? 'warning' : 'document',
    detailLines: [
      `Package: ${script.package}`,
      `Manager: ${script.packageManager}`,
      `Script: ${script.scriptType}`,
      `Risk: ${script.riskLevel}`,
      `Command: ${script.command}`,
    ],
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

export function createStatusMetrics(args: {
  summary: WorkspaceSummary;
  selectedCandidateCount: number;
  selectedCandidateBytes: number;
  auditCount: number;
}): StatusMetric[] {
  return [
    { label: 'Keep', value: formatCount(args.summary.keepCount) },
    {
      label: 'Review',
      value: `${formatCount(args.summary.reviewCount)} · ${formatBytes(args.summary.reviewBytes)}`,
    },
    {
      label: 'Queued',
      value: `${formatCount(args.selectedCandidateCount)} · ${formatBytes(args.selectedCandidateBytes)}`,
    },
    { label: 'Audit', value: formatCount(args.auditCount) },
  ];
}

export function createTotalsMetrics(args: {
  analyzedSizeBytes: number;
  safeCount: number;
  safeBytes: number;
  cleanupPoolBytes: number;
  auditCount: number;
}): TotalsMetric[] {
  return [
    { label: 'Analyzed Size', value: formatBytes(args.analyzedSizeBytes) },
    {
      label: 'Safe To Clean',
      value: `${formatCount(args.safeCount)} · ${formatBytes(args.safeBytes)}`,
    },
    { label: 'Cleanup Pool', value: formatBytes(args.cleanupPoolBytes) },
    { label: 'Audit Findings', value: formatCount(args.auditCount) },
  ];
}

export function createFooterStats(args: {
  keepCount: number;
  reviewCount: number;
  cleanupCount: number;
  auditCount: number;
}) {
  return [
    `${formatCount(args.keepCount)} keep`,
    `${formatCount(args.reviewCount)} review`,
    `${formatCount(args.cleanupCount)} cleanup`,
    `${formatCount(args.auditCount)} scripts`,
  ];
}

export function primaryActionLabel(activePane: BrowserPane, busyAction: string | null) {
  if (activePane === 'scan') {
    return busyAction === 'scan' ? 'Scanning...' : 'Refresh Scan Index';
  }
  if (activePane === 'analysis') {
    return busyAction === 'analyze' ? 'Analyzing...' : 'Refresh Analysis';
  }
  if (activePane === 'cleanup') {
    return busyAction === 'cleanup-plan' ? 'Preparing...' : 'Rebuild Cleanup Queue';
  }
  return busyAction === 'audit' ? 'Auditing...' : 'Refresh Script Audit';
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

function riskAccent(level: RiskLevel) {
  if (level === 'High') {
    return 'border-rose-400/30 bg-rose-400/10 text-rose-100';
  }
  if (level === 'Medium') {
    return 'border-amber-400/30 bg-amber-400/10 text-amber-100';
  }
  if (level === 'Low') {
    return 'border-emerald-400/30 bg-emerald-400/10 text-emerald-100';
  }
  return 'border-slate-400/20 bg-slate-400/10 text-slate-200';
}

function leafName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : path;
}
