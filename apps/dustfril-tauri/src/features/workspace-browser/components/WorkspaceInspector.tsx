import type { BrowserItem, BrowserPane, TotalsMetric } from '../model/types';
import { MetricCard } from './shared';

type WorkspaceInspectorProps = {
  activePane: BrowserPane;
  activePaneDescription: string;
  selectedItem: BrowserItem | null;
  selectedCleanupPaths: string[];
  selectedCandidateCount: number;
  totalsMetrics: TotalsMetric[];
  statusMessage: string;
  error: string | null;
  primaryActionLabel: string;
  canRunActions: boolean;
  busyAction: string | null;
  onPrimaryAction: () => void;
  onToggleCleanupSelection: () => void;
  onExecuteCleanup: () => void | Promise<void>;
};

export function WorkspaceInspector(props: WorkspaceInspectorProps) {
  const isCleanupSelectionActive =
    props.selectedItem?.path !== undefined &&
    props.selectedCleanupPaths.includes(props.selectedItem.path);

  return (
    <aside className="min-h-0 overflow-y-auto bg-[linear-gradient(180deg,#222225,#1b1b1d)] px-4 py-4">
      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Inspector</p>
        <h3 className="mt-2 text-xl font-semibold text-white">
          {props.selectedItem ? props.selectedItem.title : paneFallbackTitle(props.activePane)}
        </h3>
        <p className="mt-2 text-sm leading-6 text-slate-300">
          {props.selectedItem ? props.selectedItem.subtitle : props.activePaneDescription}
        </p>

        <div className="mt-5 grid gap-2">
          <button
            type="button"
            onClick={props.onPrimaryAction}
            disabled={!props.canRunActions}
            className="rounded-2xl bg-[#d1d1d6] px-4 py-3 text-sm font-medium text-slate-950 transition hover:bg-white disabled:cursor-not-allowed disabled:opacity-50"
          >
            {props.primaryActionLabel}
          </button>
          {props.activePane === 'cleanup' ? (
            <button
              type="button"
              onClick={props.onToggleCleanupSelection}
              disabled={!props.selectedItem?.path}
              className="rounded-2xl border border-white/10 bg-white/6 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {isCleanupSelectionActive ? 'Remove From Execution' : 'Add To Execution'}
            </button>
          ) : null}
          <button
            type="button"
            onClick={props.onExecuteCleanup}
            disabled={props.busyAction !== null || props.selectedCandidateCount === 0}
            className="rounded-2xl border border-cyan-300/20 bg-cyan-400/10 px-4 py-3 text-sm font-medium text-cyan-50 transition hover:bg-cyan-400/18 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {props.busyAction === 'cleanup-execute'
              ? 'Cleaning...'
              : `Execute Cleanup (${props.selectedCandidateCount})`}
          </button>
        </div>
      </section>

      <section className="mt-4 rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Workspace Totals</p>
        <div className="mt-4 grid gap-3">
          {props.totalsMetrics.map((metric) => (
            <MetricCard key={metric.label} label={metric.label} value={metric.value} />
          ))}
        </div>
      </section>

      <section className="mt-4 rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Execution Log</p>
        <div
          className={`mt-3 rounded-2xl border p-4 text-sm leading-6 ${
            props.error
              ? 'border-rose-400/20 bg-rose-500/10 text-rose-100'
              : 'border-emerald-400/15 bg-emerald-500/10 text-emerald-50'
          }`}
        >
          {props.statusMessage}
        </div>
      </section>
    </aside>
  );
}

function paneFallbackTitle(activePane: BrowserPane) {
  if (activePane === 'analysis') {
    return 'Artifact Library';
  }
  if (activePane === 'cleanup') {
    return 'Cleanup Queue';
  }
  if (activePane === 'scan') {
    return 'Scan Index';
  }
  return 'Script Audit';
}
