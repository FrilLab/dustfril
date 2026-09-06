import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import { EmptyState } from '../../../components/EmptyState/EmptyState';
import { FolderIcon } from '../../../components/icons';
import { formatBytes, formatCount, formatDate, formatSignedBytes } from '../../../lib/format';
import {
  changeKindLabel,
  scanExecutionLabel,
  snapshotStatusLabel,
} from '../../../model/artifactHistory';
import type {
  ActivityRecord,
  ArtifactSnapshotHistory,
  ArtifactSnapshotResult,
  ScanAccessSummary,
} from '../../../types/workflow';
import type { AsyncOperationStatus } from '../../../model/async';

type ArtifactHistoryViewProps = {
  root: string;
  history: ArtifactSnapshotHistory | null;
  status: AsyncOperationStatus;
  error: string | null;
  scanEntry: ActivityRecord | null;
  persistenceWarning: string | null;
};

export function ArtifactHistoryView(props: ArtifactHistoryViewProps) {
  return (
    <div className="artifact-history-view">
      <header className="artifact-history-heading">
        <div>
          <p className="eyebrow">Workspace</p>
          <h1>Artifact History</h1>
          <p className="heading-path" title={props.root}>
            {props.root || 'No workspace selected'}
          </p>
        </div>
      </header>

      {props.persistenceWarning ? (
        <div className="artifact-history-notice" role="status">
          {props.persistenceWarning}
        </div>
      ) : null}

      {props.status === 'loading' ? (
        <AsyncStatePanel
          status="loading"
          title="Loading artifact history"
          description="Reading retained scan summaries and generated-artifact snapshots."
        />
      ) : null}

      {props.status === 'error' ? (
        <AsyncStatePanel
          status="error"
          title="Artifact history unavailable"
          description="DustFril could not read the retained artifact snapshot state."
          error={props.error ?? undefined}
        />
      ) : null}

      {props.status === 'idle' && !props.root ? (
        <AsyncStatePanel
          status="empty"
          title="No workspace selected"
          description="Choose a workspace to view its scan summary and artifact snapshots."
        />
      ) : null}

      {props.status === 'success' && props.history ? (
        <div className="artifact-history-content">
          <ScanSummary entry={props.scanEntry} />
          <SnapshotHistory history={props.history} scanEntry={props.scanEntry} />
        </div>
      ) : null}
    </div>
  );
}

function ScanSummary({ entry }: { entry: ActivityRecord | null }) {
  return (
    <section className="artifact-history-card" aria-labelledby="scan-summary-heading">
      <div className="artifact-history-card-heading">
        <div>
          <p className="eyebrow">Bounded scan data</p>
          <h2 id="scan-summary-heading">Scan access summary</h2>
        </div>
        {entry ? (
          <span className={`history-status history-status-${entry.result.success ? 'success' : 'failed'}`}>
            {scanExecutionLabel(entry)}
          </span>
        ) : null}
      </div>

      {!entry ? (
        <EmptyState
          compact
          icon={<FolderIcon />}
          message="No scan has been run for this workspace yet. Artifact History is read-only until an explicit scan is completed."
        />
      ) : entry.result.details.accessSummary ? (
        <AccessSummaryDetails entry={entry} summary={entry.result.details.accessSummary} />
      ) : (
        <div className="artifact-history-inline-state" role="status">
          <strong>Scan summary unavailable</strong>
          <span>
            This scan record does not contain bounded access metrics. Its operation reference is{' '}
            <code>{entry.id}</code>.
          </span>
        </div>
      )}
    </section>
  );
}

function AccessSummaryDetails({
  entry,
  summary,
}: {
  entry: ActivityRecord;
  summary: ScanAccessSummary;
}) {
  return (
    <>
      <dl className="artifact-history-metrics">
        <Metric label="Scan time" value={formatDate(entry.timestampMs)} />
        <Metric label="Operation" value={entry.id} />
        <Metric label="Directories visited" value={formatCount(summary.directoriesVisited)} />
        <Metric label="Files inspected" value={formatCount(summary.filesInspected)} />
        <Metric label="Metadata files" value={formatCount(summary.metadataFilesInspected)} />
        <Metric label="Artifact candidates" value={formatCount(summary.artifactCandidates)} />
        <Metric label="Symlinks skipped" value={formatCount(summary.symlinksSkipped)} />
        <Metric label="Failures" value={formatCount(summary.failures)} />
      </dl>
      {summary.failureSamples.length ? (
        <div className="artifact-history-samples">
          <p className="eyebrow">Representative failure samples</p>
          <ul>
            {summary.failureSamples.map((failure, index) => (
              <li key={`${failure.path}-${index}`}>
                <code>{failure.path}</code>
                <span>{failure.reason}</span>
              </li>
            ))}
          </ul>
          <small>Total failures remain authoritative; displayed paths are bounded representative samples.</small>
        </div>
      ) : null}
    </>
  );
}

function SnapshotHistory({
  history,
  scanEntry,
}: {
  history: ArtifactSnapshotHistory;
  scanEntry: ActivityRecord | null;
}) {
  const entries = [...history.entries].reverse();

  return (
    <section className="artifact-history-card" aria-labelledby="snapshot-history-heading">
      <div className="artifact-history-card-heading">
        <div>
          <p className="eyebrow">Core-provided comparisons</p>
          <h2 id="snapshot-history-heading">Generated artifact snapshots</h2>
        </div>
        <span className="artifact-history-retention">
          {formatCount(history.retainedSnapshotCount)} of {formatCount(history.retentionLimit)} retained
        </span>
      </div>

      <p className="artifact-history-retention-note">
        DustFril retains at most {formatCount(history.retentionLimit)} snapshots per workspace. This view
        shows the bounded history available from Core; opening it does not create a snapshot.
      </p>

      {!history.entries.length ? (
        <EmptyState
          compact
          icon={<FolderIcon />}
          message={
            scanEntry
              ? 'No artifact snapshot is available for the latest scan. A snapshot persistence warning may explain why.'
              : 'No scan has been run yet, so there is no generated-artifact baseline to compare.'
          }
        />
      ) : (
        <div className="artifact-snapshot-list">
          {entries.map((entry) => (
            <SnapshotCard key={entry.snapshot.timestamp} entry={entry} />
          ))}
        </div>
      )}
    </section>
  );
}

function SnapshotCard({ entry }: { entry: ArtifactSnapshotResult }) {
  const isBaseline = entry.status === 'baselineCreated';

  return (
    <article className="artifact-snapshot-card">
      <header className="artifact-snapshot-header">
        <div>
          <h3>{snapshotStatusLabel(entry.status)}</h3>
          <p>{formatDate(new Date(entry.snapshot.timestamp).getTime())}</p>
        </div>
        <span className="artifact-snapshot-count">
          {formatCount(entry.snapshot.artifacts.length)} artifact{entry.snapshot.artifacts.length === 1 ? '' : 's'}
        </span>
      </header>

      {isBaseline ? (
        <p className="artifact-history-card-description">
          First retained baseline. There is no earlier snapshot for a size comparison.
        </p>
      ) : entry.changes.length ? (
        <ChangeTable entry={entry} />
      ) : (
        <p className="artifact-history-card-description">
          No generated-artifact changes were reported for this snapshot.
        </p>
      )}
    </article>
  );
}

function ChangeTable({ entry }: { entry: ArtifactSnapshotResult }) {
  return (
    <div className="artifact-change-table" role="table" aria-label="Artifact snapshot changes">
      <div className="artifact-change-row artifact-change-row-header" role="row">
        <span role="columnheader">Artifact</span>
        <span role="columnheader">State</span>
        <span role="columnheader">Previous</span>
        <span role="columnheader">Current</span>
        <span role="columnheader">Delta</span>
      </div>
      {entry.changes.map((change) => (
        <div className="artifact-change-row" role="row" key={`${change.ecosystem}:${change.path}`}>
          <span role="cell">
            <strong>{change.path}</strong>
            <small>{change.ecosystem}</small>
          </span>
          <span role="cell" className={`artifact-change-kind artifact-change-kind-${change.kind}`}>
            {changeKindLabel(change.kind)}
          </span>
          <span role="cell">{formatOptionalBytes(change.previousSizeBytes)}</span>
          <span role="cell">{formatOptionalBytes(change.currentSizeBytes)}</span>
          <span role="cell" className="artifact-change-delta">
            {formatSignedBytes(change.deltaBytes)}
          </span>
        </div>
      ))}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd title={value}>{value}</dd>
    </div>
  );
}

function formatOptionalBytes(bytes: number | null) {
  return bytes === null ? '—' : formatBytes(bytes);
}
