import type { CleanupHistoryEntry } from '../../types/workflow';
import { formatBytes, formatDate } from '../../lib/format';
import { EmptyState } from '../EmptyState/EmptyState';

type CleanupHistoryProps = {
  entries: CleanupHistoryEntry[];
};

export function CleanupHistory(props: CleanupHistoryProps) {
  if (!props.entries.length) {
    return <EmptyState message="No cleanup history yet. Completed cleanups will appear here." />;
  }

  return (
    <div className="space-y-4">
      {props.entries.map((entry, index) => (
        <article
          key={`${entry.executedAtMs}-${index}`}
          className="rounded-[24px] border border-white/8 bg-black/12 p-4"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-sm font-semibold text-white">{formatDate(entry.executedAtMs)}</p>
              <p className="mt-1 text-xs text-slate-400">Mode: {entry.mode}</p>
            </div>
            <p className="rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs text-emerald-100">
              Freed {formatBytes(entry.freedSizeBytes)}
            </p>
          </div>

          {entry.deletedPaths.length ? (
            <div className="mt-4">
              <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Deleted</p>
              <ul className="mt-2 space-y-1 text-sm text-slate-300">
                {entry.deletedPaths.map((path) => (
                  <li key={path} className="truncate">
                    - {path}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          {entry.failedPaths.length ? (
            <div className="mt-4">
              <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Failed</p>
              <ul className="mt-2 space-y-1 text-sm text-rose-200">
                {entry.failedPaths.map((path) => (
                  <li key={path} className="truncate">
                    - {path}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </article>
      ))}
    </div>
  );
}
