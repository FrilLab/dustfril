import { formatAge, formatBytes } from '../lib/format';
import type { CleanupPlanResponse, CleanupResultResponse } from '../types/workflow';

type CleanupSectionProps = {
  cleanupPlan: CleanupPlanResponse | null;
  cleanupResult: CleanupResultResponse | null;
  busyAction: string | null;
  selectedCleanupPaths: string[];
  selectedCandidateCount: number;
  selectedCandidateBytes: number;
  onToggleCleanupPath: (path: string) => void;
  onExecute: () => void | Promise<void>;
};

export function CleanupSection(props: CleanupSectionProps) {
  return (
    <div className="rounded-[32px] border border-white/10 bg-[linear-gradient(180deg,rgba(251,146,60,0.12),rgba(15,23,42,0.72))] p-6 shadow-[0_20px_80px_rgba(15,23,42,0.3)] backdrop-blur md:p-8">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.24em] text-orange-100/70">
            Cleanup
          </p>
          <h3 className="mt-2 text-2xl font-semibold text-white">Execution queue</h3>
        </div>
        <button
          type="button"
          onClick={props.onExecute}
          disabled={props.busyAction !== null || props.selectedCandidateCount === 0}
          className="rounded-full bg-white px-4 py-2 text-sm font-medium text-slate-950 transition hover:bg-orange-100 disabled:cursor-not-allowed disabled:opacity-60"
        >
          {props.busyAction === 'cleanup-execute' ? 'Cleaning...' : 'Execute'}
        </button>
      </div>

      <div className="mt-5 rounded-3xl border border-white/10 bg-black/15 p-4">
        <p className="text-sm text-slate-300">Selected candidates</p>
        <p className="mt-2 text-2xl font-semibold text-white">{props.selectedCandidateCount}</p>
        <p className="mt-1 text-sm text-slate-200">{formatBytes(props.selectedCandidateBytes)}</p>
      </div>

      <div className="mt-5 space-y-3">
        {props.cleanupPlan?.candidates.length ? (
          props.cleanupPlan.candidates.map((candidate) => (
            <label key={candidate.path} className="flex cursor-pointer gap-3 rounded-3xl border border-white/10 bg-black/15 p-4">
              <input
                type="checkbox"
                checked={props.selectedCleanupPaths.includes(candidate.path)}
                onChange={() => props.onToggleCleanupPath(candidate.path)}
                className="mt-1 h-4 w-4 rounded border-white/20 bg-slate-950/50"
              />
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="rounded-full border border-cyan-400/25 bg-cyan-400/10 px-3 py-1 text-xs text-cyan-100">
                    {candidate.ecosystem}
                  </span>
                  <span className="text-sm text-slate-200">{formatBytes(candidate.sizeBytes)}</span>
                </div>
                <p className="mt-2 break-all text-sm font-medium text-white">{candidate.path}</p>
                <p className="mt-1 text-sm text-slate-300">{formatAge(candidate.ageDays)}</p>
              </div>
            </label>
          ))
        ) : (
          <div className="rounded-3xl border border-dashed border-white/10 bg-black/10 p-6 text-sm text-slate-300">
            No cleanup plan loaded.
          </div>
        )}
      </div>

      {props.cleanupResult ? (
        <div className="mt-5 rounded-3xl border border-emerald-400/20 bg-emerald-500/10 p-4">
          <p className="text-sm font-medium text-white">Last cleanup result</p>
          <p className="mt-2 text-sm text-emerald-100">
            Freed {formatBytes(props.cleanupResult.freedSizeBytes)} from{' '}
            {props.cleanupResult.deletedPaths.length} path(s).
          </p>
          {props.cleanupResult.failedPaths.length ? (
            <div className="mt-3 space-y-2 text-sm text-rose-100">
              {props.cleanupResult.failedPaths.map((failure) => (
                <p key={`${failure.path}-${failure.reason}`}>
                  {failure.reason}: {failure.path}
                </p>
              ))}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
