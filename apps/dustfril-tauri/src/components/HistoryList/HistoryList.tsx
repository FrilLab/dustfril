import type { ActivityRecord } from '../../types/workflow';
import { formatBytes, formatDate } from '../../lib/format';
import { EmptyState } from '../EmptyState/EmptyState';

type HistoryListProps = {
  entries: ActivityRecord[];
};

export function HistoryList(props: HistoryListProps) {
  if (!props.entries.length) {
    return <EmptyState message="No activity history yet. Completed operations will appear here." />;
  }

  return (
    <div className="space-y-4">
      {props.entries.map((entry, index) => (
        <article
          key={`${entry.id}-${index}`}
          className="rounded-[24px] border border-white/8 bg-black/12 p-4"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-sm font-semibold text-white">
                {entry.kind} · {formatDate(entry.timestampMs)}
              </p>
              <p className="mt-1 text-xs text-slate-400">
                Result: {entry.result.success ? 'Succeeded' : 'Failed'}
              </p>
            </div>
            <p
              className={`rounded-full border px-3 py-1 text-xs ${
                entry.result.success
                  ? 'border-emerald-400/20 bg-emerald-400/10 text-emerald-100'
                  : 'border-rose-400/20 bg-rose-400/10 text-rose-100'
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
    <div className="mt-4 grid gap-2 text-sm text-slate-300 sm:grid-cols-3">
      <Detail label="Target" value={details.path ?? 'Unknown'} />
      <Detail label="Artifacts" value={String(details.artifacts ?? 0)} />
      <Detail label="Total Size" value={formatBytes(details.size ?? 0)} />
    </div>
  );
}

function CleanupDetails({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;
  const deleted = details.deleted ?? [];
  const failed = details.failed ?? [];

  return (
    <>
      <div className="mt-4 grid gap-2 text-sm text-slate-300 sm:grid-cols-2">
        <Detail label="Mode" value={details.mode ?? 'Unknown'} />
        <Detail label="Freed" value={formatBytes(details.freed ?? 0)} />
      </div>

      {deleted.length ? (
        <PathList label="Deleted" paths={deleted} />
      ) : null}

      {failed.length ? (
        <div className="mt-4">
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Failed</p>
          <ul className="mt-2 space-y-1 text-sm text-rose-200">
            {failed.map((failure) => (
              <li key={`${failure.path}-${failure.reason ?? ''}`} className="truncate">
                - {failure.path}
                {failure.reason ? ` (${failure.reason})` : ''}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </>
  );
}

function SecurityDetails({ entry }: { entry: ActivityRecord }) {
  const details = entry.result.details;
  const findings = details.findings ?? [];

  return (
    <>
      <div className="mt-4 grid gap-2 text-sm text-slate-300 sm:grid-cols-4">
        <Detail label="Target" value={details.path ?? 'Unknown'} />
        <Detail label="Ecosystems" value={details.ecosystems?.join(', ') ?? 'All'} />
        <Detail label="Findings" value={String(details.findingCount ?? findings.length)} />
        <Detail label="Highest Risk" value={details.highestRisk ?? 'None'} />
      </div>

      {details.reason ? (
        <p className="mt-4 text-sm text-rose-200">{details.reason}</p>
      ) : null}

      {findings.length ? (
        <div className="mt-4">
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Findings</p>
          <ul className="mt-2 space-y-2 text-sm text-slate-300">
            {findings.map((finding, index) => (
              <li
                key={`${finding.rule}-${finding.source}-${index}`}
                className="rounded-2xl border border-white/6 bg-white/4 px-3 py-2"
              >
                <p className="font-medium text-slate-200">
                  {finding.rule} · {finding.risk}
                </p>
                <p className="mt-1 truncate text-xs text-slate-400">{finding.source}</p>
                <p className="mt-1 text-xs text-slate-300">{finding.reason}</p>
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
    <div className="mt-4">
      <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{label}</p>
      <ul className="mt-2 space-y-1 text-sm text-slate-300">
        {paths.map((path) => (
          <li key={path} className="truncate">
            - {path}
          </li>
        ))}
      </ul>
    </div>
  );
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-white/6 bg-white/4 px-3 py-2">
      <p className="text-xs uppercase tracking-[0.14em] text-slate-500">{label}</p>
      <p className="mt-1 truncate text-slate-200">{value}</p>
    </div>
  );
}
