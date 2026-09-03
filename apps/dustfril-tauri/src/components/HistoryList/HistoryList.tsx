import { useEffect, useState } from 'react';
import type {
  ActivityDetails,
  ActivityRecord,
  CleanupActivityItem,
  ScanAccessSummary,
  SecurityActivityFinding,
} from '../../types/workflow';
import { formatBytes, formatDate } from '../../lib/format';
import { leafName } from '../../model/presentation';
import { EmptyState } from '../EmptyState/EmptyState';

type HistoryListProps = {
  entries: ActivityRecord[];
};

export function HistoryList(props: HistoryListProps) {
  const [selectedEntryId, setSelectedEntryId] = useState<string | null>(null);
  const selectedEntry =
    props.entries.find((entry) => entry.id === selectedEntryId) ?? null;

  useEffect(() => {
    if (selectedEntryId && !selectedEntry) {
      setSelectedEntryId(null);
    }
  }, [selectedEntry, selectedEntryId]);

  useEffect(() => {
    if (!selectedEntry) {
      return;
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setSelectedEntryId(null);
      }
    }

    window.addEventListener('keydown', closeOnEscape);
    return () => window.removeEventListener('keydown', closeOnEscape);
  }, [selectedEntry]);

  if (!props.entries.length) {
    return <EmptyState message="No activity history yet. Operations will appear here." />;
  }

  return (
    <div className="history-list-container">
      <div className="history-table" role="table" aria-label="Activity history">
        <div className="history-row history-row-header" role="row">
          <span role="columnheader">Time</span>
          <span role="columnheader">Action</span>
          <span role="columnheader">Target</span>
          <span role="columnheader">Result</span>
          <span role="columnheader">Status</span>
          <span aria-hidden="true" />
        </div>
        <div className="history-table-body">
          {props.entries.map((entry, index) => (
            <HistoryRow
              key={`${entry.id}-${index}`}
              entry={entry}
              selected={entry.id === selectedEntryId}
              onSelect={() => setSelectedEntryId(entry.id)}
            />
          ))}
        </div>
      </div>

      {selectedEntry ? (
        <HistoryDetailsDrawer
          entry={selectedEntry}
          onClose={() => setSelectedEntryId(null)}
        />
      ) : null}
    </div>
  );
}

function HistoryRow({
  entry,
  selected,
  onSelect,
}: {
  entry: ActivityRecord;
  selected: boolean;
  onSelect: () => void;
}) {
  const status = historyStatusLabel(entry);

  return (
    <button
      type="button"
      className={`history-row history-row-button${selected ? ' history-row-active' : ''}`}
      onClick={onSelect}
      aria-label={`Inspect ${entry.kind} activity for ${historyTargetLabel(entry)}`}
    >
      <span className="history-time">{formatHistoryDate(entry.timestampMs)}</span>
      <span className={`history-kind history-kind-${entry.kind.toLowerCase()}`}>
        <span className="history-kind-mark" aria-hidden="true" />
        {entry.kind}
      </span>
      <span className="history-target" title={historyTargetLabel(entry)}>
        {historyTargetLabel(entry)}
      </span>
      <span className="history-result" title={historyResultLabel(entry)}>
        {historyResultLabel(entry)}
      </span>
      <span className={`history-status ${historyStatusClass(status)}`}>{status}</span>
      <span className="history-chevron" aria-hidden="true">
        ›
      </span>
    </button>
  );
}

function HistoryDetailsDrawer({
  entry,
  onClose,
}: {
  entry: ActivityRecord;
  onClose: () => void;
}) {
  const title = `${entry.kind} details`;
  const status = historyStatusLabel(entry);

  return (
    <aside className="inspector-pane workspace-drawer history-drawer" aria-label="History details">
      <div className="inspector-header">
        <span className="eyebrow">History details</span>
        <button
          type="button"
          className="inspector-close"
          onClick={onClose}
          aria-label="Close history details"
        >
          ×
        </button>
      </div>
      <div className="inspector-content">
        <div className="history-drawer-title">
          <span className={`history-kind history-kind-${entry.kind.toLowerCase()}`}>
            <span className="history-kind-mark" aria-hidden="true" />
            {entry.kind}
          </span>
          <h2>{title}</h2>
          <p>{formatDate(entry.timestampMs)}</p>
          <span className={`history-status ${historyStatusClass(status)}`}>{status}</span>
        </div>

        {entry.kind === 'Scan' ? <ScanDetails details={entry.result.details} /> : null}
        {entry.kind === 'Cleanup' ? <CleanupDetails details={entry.result.details} /> : null}
        {entry.kind === 'Security' ? <SecurityDetails details={entry.result.details} /> : null}
      </div>
    </aside>
  );
}

function ScanDetails({ details }: { details: ActivityDetails }) {
  const summary = details.accessSummary;

  return (
    <>
      <dl className="inspector-details history-drawer-details">
        <Detail label="Target" value={details.path ?? 'Unknown'} />
        <Detail label="Artifacts" value={String(details.artifacts ?? 0)} />
        <Detail label="Total size" value={formatBytes(details.size ?? 0)} />
      </dl>
      {details.reason ? <FailureReason value={details.reason} /> : null}
      {summary ? <ScanAccessDetails summary={summary} /> : null}
    </>
  );
}

function ScanAccessDetails({ summary }: { summary: ScanAccessSummary }) {
  return (
    <section className="history-drawer-section" aria-labelledby="scan-access-heading">
      <p className="eyebrow" id="scan-access-heading">
        Scan access
      </p>
      <dl className="inspector-details history-drawer-details history-access-details">
        <Detail label="Directories" value={String(summary.directoriesVisited)} />
        <Detail label="Files inspected" value={String(summary.filesInspected)} />
        <Detail label="Metadata files" value={String(summary.metadataFilesInspected)} />
        <Detail label="Artifact candidates" value={String(summary.artifactCandidates)} />
        <Detail label="Symlinks skipped" value={String(summary.symlinksSkipped)} />
        <Detail label="Failures" value={String(summary.failures)} />
      </dl>
      {summary.failureSamples.length ? (
        <FailureList
          label="Representative failures"
          items={summary.failureSamples.map((failure) => `${failure.path}: ${failure.reason}`)}
        />
      ) : null}
    </section>
  );
}

function CleanupDetails({ details }: { details: ActivityDetails }) {
  const items = cleanupItems(details);
  const succeededCount = items.filter((item) => item.status === 'succeeded').length;
  const failedCount = items.filter((item) => item.status === 'failed').length;
  const affectedCount = succeededCount + failedCount;

  return (
    <>
      <dl className="inspector-details history-drawer-details">
        {details.target ? <Detail label="Target / workspace" value={details.target} /> : null}
        <Detail label="Mode" value={cleanupModeLabel(details.mode)} />
        <Detail label="Items" value={String(affectedCount)} />
        <Detail label="Size" value={formatBytes(details.freed ?? 0)} />
        <Detail
          label="Result"
          value={`${succeededCount} succeeded · ${failedCount} failed`}
        />
      </dl>

      {items.length ? (
        <section className="history-drawer-section" aria-labelledby="cleanup-items-heading">
          <p className="eyebrow" id="cleanup-items-heading">
            Affected targets
          </p>
          <div className="history-cleanup-items">
            {items.map((item, index) => (
              <CleanupItem item={item} mode={details.mode} key={`${item.path}-${index}`} />
            ))}
          </div>
        </section>
      ) : null}

      {details.reason ? <FailureReason value={details.reason} /> : null}
    </>
  );
}

function CleanupItem({
  item,
  mode,
}: {
  item: CleanupActivityItem;
  mode: ActivityDetails['mode'];
}) {
  const outcome = item.status === 'succeeded'
    ? mode === 'trash'
      ? 'Moved to Trash'
      : 'Deleted permanently'
    : `Failed${item.reason ? `: ${item.reason}` : ''}`;

  return (
    <div className="history-cleanup-item">
      <div className="min-width-zero">
        <strong>{item.project || leafName(item.path)}</strong>
        <span title={item.path}>{item.path}</span>
      </div>
      <div className="history-cleanup-item-meta">
        {item.size !== undefined ? <span>{formatBytes(item.size)}</span> : null}
        <span className={item.status === 'failed' ? 'history-item-failed' : 'history-item-success'}>
          {outcome}
        </span>
      </div>
    </div>
  );
}

function SecurityDetails({ details }: { details: ActivityDetails }) {
  const findings = details.findings ?? [];

  return (
    <>
      <dl className="inspector-details history-drawer-details">
        <Detail label="Target" value={details.path ?? 'Unknown'} />
        <Detail
          label="Ecosystems"
          value={details.ecosystems?.length ? details.ecosystems.join(', ') : 'All'}
        />
        <Detail label="Findings" value={String(details.findingCount ?? findings.length)} />
        <Detail label="Highest risk" value={details.highestRisk ?? 'None'} />
      </dl>

      {details.reason ? <FailureReason value={details.reason} /> : null}
      {findings.length ? <SecurityFindingList findings={findings} /> : null}
    </>
  );
}

function SecurityFindingList({ findings }: { findings: SecurityActivityFinding[] }) {
  return (
    <section className="history-drawer-section" aria-labelledby="history-findings-heading">
      <p className="eyebrow" id="history-findings-heading">
        Findings
      </p>
      <ul className="history-finding-list">
        {findings.map((finding, index) => (
          <li key={`${finding.rule}-${finding.source}-${index}`} className="history-finding">
            <p className="history-finding-title">
              {finding.rule} · {finding.risk}
            </p>
            <p className="history-finding-source">{finding.source}</p>
            <p className="history-finding-reason">{finding.reason}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}

function FailureList({ label, items }: { label: string; items: string[] }) {
  return (
    <div className="history-failure-list">
      <p className="eyebrow">{label}</p>
      <ul>
        {items.map((item, index) => (
          <li key={`${item}-${index}`}>{item}</li>
        ))}
      </ul>
    </div>
  );
}

function FailureReason({ value }: { value: string }) {
  return <p className="history-detail-reason">{value}</p>;
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd title={value}>{value}</dd>
    </div>
  );
}

function cleanupItems(details: ActivityDetails): CleanupActivityItem[] {
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

function cleanupModeLabel(mode: ActivityDetails['mode']) {
  return mode === 'trash'
    ? 'Move to Trash'
    : mode === 'permanent'
      ? 'Delete permanently'
      : 'Unknown';
}

export function historyStatusLabel(entry: ActivityRecord) {
  if (historyFailureCount(entry) > 0) {
    return 'Partial failure';
  }

  return entry.result.success ? 'Success' : 'Failed';
}

function historyStatusClass(status: string) {
  return status === 'Success' ? 'history-status-success' : 'history-status-failure';
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

function historyTargetLabel(entry: ActivityRecord) {
  const details = entry.result.details;

  if (entry.kind === 'Cleanup') {
    const projects = Array.from(
      new Set(
        (details.items ?? [])
          .map((item) => item.project)
          .filter((project): project is string => Boolean(project)),
      ),
    );
    if (projects.length === 1) {
      return projects[0];
    }
    if (projects.length > 1) {
      return `${projects[0]} + ${projects.length - 1}`;
    }
  }

  return conciseTargetName(details.target ?? details.path ?? firstCleanupPath(details));
}

function firstCleanupPath(details: ActivityDetails) {
  return details.deleted?.[0] ?? details.failed?.[0]?.path ?? 'Unknown';
}

function conciseTargetName(path: string) {
  return path === 'Unknown' ? path : leafName(path);
}

function historyResultLabel(entry: ActivityRecord) {
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

function formatHistoryDate(timestampMs: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date(timestampMs));
}
