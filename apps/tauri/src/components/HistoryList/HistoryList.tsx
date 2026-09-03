import { useEffect, useMemo, useState } from 'react';
import { SortableHeader } from '../SortableHeader/SortableHeader';
import type {
  ActivityDetails,
  ActivityRecord,
  CleanupActivityItem,
  ScanAccessSummary,
  SecurityActivityFinding,
} from '../../types/workflow';
import { formatBytes, formatDate } from '../../lib/format';
import {
  cleanupItems,
  cleanupModeLabel,
  historyResultLabel,
  historyStatusLabel,
  historyTargetLabel,
} from '../../model/activity';
import { leafName } from '../../model/presentation';
import { sortActivityRecords, type HistorySortColumn, type HistorySortState } from '../../model/sorting';

export { historyStatusLabel } from '../../model/activity';

type HistoryListProps = {
  entries: ActivityRecord[];
};

export function HistoryList(props: HistoryListProps) {
  const [selectedEntryId, setSelectedEntryId] = useState<string | null>(null);
  const [sort, setSort] = useState<HistorySortState>({ column: 'time', direction: 'desc' });
  const selectedEntry =
    props.entries.find((entry) => entry.id === selectedEntryId) ?? null;
  const sortedEntries = useMemo(
    () => sortActivityRecords(props.entries, sort),
    [props.entries, sort],
  );

  function handleSort(column: HistorySortColumn) {
    setSort((current) => ({
      column,
      direction: current.column === column && current.direction === 'asc' ? 'desc' : 'asc',
    }));
  }

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
    return (
      <div className="history-list-layout history-list-empty">
        <div className="empty-state history-empty-copy" aria-live="polite">
          <p>No activity yet</p>
          <p>Scans and cleanup operations will appear here.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="history-list-layout">
      <div className="history-list-scroll">
        <div className="history-table" role="table" aria-label="Activity history">
          <div className="history-row history-row-header" role="row">
            <SortableHeader
              label="Time"
              active={sort.column === 'time'}
              direction={sort.direction}
              onSort={() => handleSort('time')}
            />
            <SortableHeader
              label="Action"
              active={sort.column === 'action'}
              direction={sort.direction}
              onSort={() => handleSort('action')}
            />
            <SortableHeader
              label="Target"
              active={sort.column === 'target'}
              direction={sort.direction}
              onSort={() => handleSort('target')}
            />
            <SortableHeader
              label="Result"
              active={sort.column === 'result'}
              direction={sort.direction}
              onSort={() => handleSort('result')}
            />
            <SortableHeader
              label="Status"
              active={sort.column === 'status'}
              direction={sort.direction}
              onSort={() => handleSort('status')}
            />
            <span aria-hidden="true" />
          </div>
          <div className="history-table-body">
            {sortedEntries.map((entry) => (
              <HistoryRow
                key={entry.id}
                entry={entry}
                selected={entry.id === selectedEntryId}
                onSelect={() => setSelectedEntryId(entry.id)}
              />
            ))}
          </div>
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
    <div
      className={`history-row history-row-interactive${selected ? ' history-row-active' : ''}`}
      role="row"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onSelect();
        }
      }}
      aria-label={`Inspect ${entry.kind} activity for ${historyTargetLabel(entry)}`}
    >
      <span className="history-time" role="cell">
        {formatHistoryDate(entry.timestampMs)}
      </span>
      <span className={`history-kind history-kind-${entry.kind.toLowerCase()}`} role="cell">
        <span className="history-kind-mark" aria-hidden="true" />
        {entry.kind}
      </span>
      <span className="history-target" role="cell" title={historyTargetLabel(entry)}>
        {historyTargetLabel(entry)}
      </span>
      <span className="history-result" role="cell" title={historyResultLabel(entry)}>
        {historyResultLabel(entry)}
      </span>
      <span className={`history-status ${historyStatusClass(status)}`} role="cell">
        {status}
      </span>
      <span className="history-chevron" role="cell" aria-hidden="true">
        ›
      </span>
    </div>
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
        <Detail label="Ecosystems" value={securityEcosystemLabel(details.ecosystems)} />
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

function historyStatusClass(status: string) {
  return status === 'Success' ? 'history-status-success' : 'history-status-failure';
}

function securityEcosystemLabel(ecosystems: string[] | undefined) {
  if (ecosystems === undefined) {
    return 'All';
  }

  return ecosystems.length ? ecosystems.join(', ') : 'None';
}


function formatHistoryDate(timestampMs: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date(timestampMs));
}
