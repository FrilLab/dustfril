import { useMemo } from 'react';
import { EmptyState } from '../../../components/EmptyState/EmptyState';
import { formatBytes, formatCount, formatDate } from '../../../lib/format';
import { cleanupItemCount, cleanupModeLabel, historyStatusLabel } from '../../../model/activity';
import { leafName, recommendationClass, recommendationLabel } from '../../../model/presentation';
import { sortArtifacts } from '../../../model/sorting';
import type { ActivityRecord, ArtifactAnalysis, CleanupCandidate } from '../../../types/workflow';

type OverviewViewProps = {
  root: string;
  analysisReady: boolean;
  artifacts: ArtifactAnalysis[];
  candidates: CleanupCandidate[];
  reclaimableBytes: number;
  historyEntries: ActivityRecord[];
  error: string | null;
  onInspectArtifact: (path: string) => void;
  onOpenHistory: () => void;
};

export function OverviewView(props: OverviewViewProps) {
  const reclaimableCandidates = props.candidates.filter(
    (candidate) => candidate.selectedByDefault,
  );
  const needsReviewArtifacts = props.artifacts.filter(
    (artifact) => artifact.recommendation === 'NeedsReview',
  );
  const largestArtifacts = useMemo(
    () => sortArtifacts(props.artifacts, { column: 'size', direction: 'desc' }).slice(0, 5),
    [props.artifacts],
  );
  const lastCleanup = useMemo(
    () =>
      props.historyEntries
        .filter((entry) => entry.kind === 'Cleanup')
        .reduce<ActivityRecord | null>(
          (latest, entry) =>
            latest === null ||
            entry.timestampMs > latest.timestampMs ||
            (entry.timestampMs === latest.timestampMs && entry.id > latest.id)
              ? entry
              : latest,
          null,
        ),
    [props.historyEntries],
  );
  const needsReviewBytes = needsReviewArtifacts.reduce(
    (total, artifact) => total + artifact.sizeBytes,
    0,
  );

  return (
    <div className="overview-view">
      <div className="overview-intro">
        <p className="eyebrow">Overview</p>
        <h1>{props.root ? leafName(props.root) : 'Overview'}</h1>
        <p className="overview-heading-path" title={props.root}>
          {props.root || 'No workspace selected'}
        </p>
      </div>

      <div className="overview-summary-grid">
        <OverviewSummaryCard
          label="Reclaimable now"
          bytes={props.reclaimableBytes}
          count={reclaimableCandidates.length}
          analysisReady={props.analysisReady}
        />
        <OverviewSummaryCard
          label="Needs Review"
          bytes={needsReviewBytes}
          count={needsReviewArtifacts.length}
          analysisReady={props.analysisReady}
        />
      </div>

      {!props.analysisReady ? (
        <p className="overview-analysis-hint" role="status">
          Analyze this workspace to see cleanup insights.
        </p>
      ) : null}

      {props.error ? (
        <div className="overview-notice overview-notice-warning" role="status">
          {props.error}
        </div>
      ) : null}

      <section className="overview-panel" aria-labelledby="largest-artifacts-heading">
        <div className="overview-section-heading">
          <div>
            <p className="eyebrow" id="largest-artifacts-heading">
              Largest artifacts
            </p>
            <p className="overview-caption">
              The biggest normalized artifacts found in this workspace.
            </p>
          </div>
        </div>
        {!props.analysisReady ? (
          <EmptyState compact message="Analyze this workspace to load artifacts." />
        ) : largestArtifacts.length ? (
          <div className="overview-artifact-table" role="table" aria-label="Largest artifacts">
            <div className="overview-artifact-row overview-artifact-row-header" role="row">
              <span role="columnheader">Project</span>
              <span role="columnheader">Artifact</span>
              <span role="columnheader">Size</span>
              <span role="columnheader">Status</span>
            </div>
            {largestArtifacts.map((artifact) => (
              <button
                type="button"
                className="overview-artifact-row overview-artifact-row-button"
                key={artifact.path}
                onClick={() => props.onInspectArtifact(artifact.path)}
                aria-label={`Inspect ${artifact.project.displayName} ${leafName(artifact.path)}`}
              >
                <span>{artifact.project.displayName}</span>
                <span>{leafName(artifact.path)}</span>
                <span>{formatBytes(artifact.sizeBytes)}</span>
                <span className={recommendationClass(artifact.recommendation)}>
                  {recommendationLabel(artifact.recommendation)}
                </span>
              </button>
            ))}
          </div>
        ) : (
          <EmptyState compact message="No supported development artifacts were found." />
        )}
      </section>

      <section
        className="overview-panel overview-last-cleanup"
        aria-labelledby="last-cleanup-heading"
      >
        <div className="overview-section-heading">
          <p className="eyebrow" id="last-cleanup-heading">
            Last cleanup
          </p>
          <button type="button" className="overview-link" onClick={props.onOpenHistory}>
            View history
          </button>
        </div>
        {lastCleanup ? (
          <LastCleanupSummary entry={lastCleanup} />
        ) : (
          <p className="overview-muted">No cleanup operations yet.</p>
        )}
      </section>
    </div>
  );
}

function OverviewSummaryCard({
  label,
  bytes,
  count,
  analysisReady,
}: {
  label: string;
  bytes: number;
  count: number;
  analysisReady: boolean;
}) {
  return (
    <article className="overview-summary-card">
      <p className="eyebrow">{label}</p>
      {analysisReady ? (
        <>
          <strong>{formatBytes(bytes)}</strong>
          <span>
            {formatCount(count)} artifact{count === 1 ? '' : 's'}
          </span>
        </>
      ) : (
        <strong className="overview-summary-unavailable">Not analyzed</strong>
      )}
    </article>
  );
}

function LastCleanupSummary({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;
  const itemCount = cleanupItemCount(details);

  return (
    <div className="last-cleanup-summary">
      <strong>{formatDate(entry.timestampMs)}</strong>
      <span>
        {formatCount(itemCount)} artifact{itemCount === 1 ? '' : 's'} · {formatBytes(details.freed ?? 0)}
      </span>
      <span>
        {cleanupModeLabel(details.mode)} · {historyStatusLabel(entry)}
      </span>
    </div>
  );
}
