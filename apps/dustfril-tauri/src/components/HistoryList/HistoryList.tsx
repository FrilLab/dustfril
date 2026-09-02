import type { ActivityDetails, ActivityRecord } from '../../types/workflow';
import { formatBytes, formatDate } from '../../lib/format';
import { EmptyState } from '../EmptyState/EmptyState';

type HistoryListProps = {
  entries: ActivityRecord[];
};

export function HistoryList(props: HistoryListProps) {
  if (!props.entries.length) {
    return <EmptyState message="No activity history yet. Operations will appear here." />;
  }

  return (
    <div className="history-list">
      {props.entries.map((entry, index) => (
        <article
          key={`${entry.id}-${index}`}
          className="history-entry"
        >
          <div className="history-entry-header">
            <div>
              <p className="history-entry-title">
                {entry.kind} · {formatDate(entry.timestampMs)}
              </p>
              <p className="history-entry-subtitle">
                Result: {entry.result.success ? 'Succeeded' : 'Failed'}
              </p>
            </div>
            <p
              className={`history-badge ${
                entry.result.success
                  ? 'history-badge-success'
                  : 'history-badge-failure'
              }`}
            >
              {entry.result.success ? 'Success' : 'Failure'}
            </p>
          </div>

          {entry.kind === 'Scan' ? <ScanDetails entry={entry} /> : null}
          {entry.kind === 'Cleanup' ? <CleanupDetails entry={entry} /> : null}
          {entry.kind === 'Security' ? <SecurityDetails entry={entry} /> : null}
        </article>
      ))}
    </div>
  );
}

function ScanDetails({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;

  return (
    <>
      <div className="history-details-grid">
        <Detail label="Target" value={details.path ?? 'Unknown'} />
        <Detail label="Artifacts" value={String(details.artifacts ?? 0)} />
        <Detail label="Total Size" value={formatBytes(details.size ?? 0)} />
      </div>
      {details.accessSummary ? <ScanAccessDetails summary={details.accessSummary} /> : null}
      {details.reason ? <p className="mt-4 text-sm text-rose-200">{details.reason}</p> : null}
    </>
  );
}

function ScanAccessDetails({
  summary,
}: {
  summary: NonNullable<ActivityDetails['accessSummary']>;
}) {
  return (
    <div className="history-access-summary">
      <p className="eyebrow">Scan access summary</p>
      <div className="history-details-grid">
        <Detail label="Directories" value={String(summary.directoriesVisited)} />
        <Detail label="Files Inspected" value={String(summary.filesInspected)} />
        <Detail label="Metadata Files" value={String(summary.metadataFilesInspected)} />
        <Detail label="Artifact Candidates" value={String(summary.artifactCandidates)} />
        <Detail label="Symlinks Skipped" value={String(summary.symlinksSkipped)} />
        <Detail label="Failures" value={String(summary.failures)} />
      </div>
      {summary.failureSamples.length ? (
        <div className="history-failure-list">
          <p className="eyebrow">Representative failures</p>
          <ul>
            {summary.failureSamples.map((failure) => (
            <li key={`${failure.path}-${failure.reason}`}>
                {failure.path}: {failure.reason}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}

function CleanupDetails({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;
  const deleted = details.deleted ?? [];
  const failed = details.failed ?? [];

  return (
    <>
      <div className="history-details-grid">
        <Detail label="Mode" value={details.mode ?? 'Unknown'} />
        <Detail label="Freed" value={formatBytes(details.freed ?? 0)} />
      </div>

      {deleted.length ? (
        <PathList label="Deleted" paths={deleted} />
      ) : null}

      {failed.length ? (
      <div className="history-failure-list">
          <p className="eyebrow">Failed</p>
          <ul>
            {failed.map((failure) => (
              <li key={`${failure.path}-${failure.reason ?? ''}`}>
                - {failure.path}
                {failure.reason ? ` (${failure.reason})` : ''}
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {details.reason ? <p className="mt-4 text-sm text-rose-200">{details.reason}</p> : null}
    </>
  );
}

function SecurityDetails({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;
  const findings = details.findings ?? [];

  return (
    <>
      <div className="history-details-grid">
        <Detail label="Target" value={details.path ?? 'Unknown'} />
        <Detail
          label="Ecosystems"
          value={
            details.ecosystems
              ? details.ecosystems.length
                ? details.ecosystems.join(', ')
                : 'None'
              : 'All'
          }
        />
        <Detail label="Findings" value={String(details.findingCount ?? findings.length)} />
        <Detail label="Highest Risk" value={details.highestRisk ?? 'None'} />
      </div>

      {details.reason ? (
        <p className="mt-4 text-sm text-rose-200">{details.reason}</p>
      ) : null}

      {findings.length ? (
        <div className="history-finding-list">
          <p className="eyebrow">Findings</p>
          <ul>
            {findings.map((finding, index) => (
              <li
                key={`${finding.rule}-${finding.source}-${index}`}
                className="history-finding"
              >
                <p className="history-finding-title">
                  {finding.rule} · {finding.risk}
                </p>
                <p className="history-finding-source">{finding.source}</p>
                <p className="history-finding-reason">{finding.reason}</p>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </>
  );
}

function PathList({ label, paths }: { label: string; paths: string[] }) {
  return (
    <div className="history-failure-list">
      <p className="eyebrow">{label}</p>
      <ul>
        {paths.map((path) => (
          <li key={path}>
            - {path}
          </li>
        ))}
      </ul>
    </div>
  );
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div className="history-detail">
      <dt>{label}</dt>
      <dd title={value}>{value}</dd>
    </div>
  );
}
