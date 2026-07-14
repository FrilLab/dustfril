import type { DeleteMode, Ecosystem } from '../types/workflow';
import { deleteModes, ecosystems } from '../types/workflow';

type ControlPanelProps = {
  root: string;
  selectedEcosystems: Ecosystem[];
  deleteMode: DeleteMode;
  busyAction: string | null;
  onRootChange: (value: string) => void;
  onToggleEcosystem: (ecosystem: Ecosystem) => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  onScan: () => void | Promise<void>;
  onAnalyze: () => void | Promise<void>;
  onBuildCleanupPlan: () => void | Promise<void>;
  onAudit: () => void | Promise<void>;
};

export function ControlPanel(props: ControlPanelProps) {
  const disabled = () => props.busyAction !== null || props.selectedEcosystems.length === 0;

  return (
    <div className="rounded-[32px] border border-white/10 bg-white/6 p-6 shadow-[0_20px_80px_rgba(15,23,42,0.3)] backdrop-blur md:p-8">
      <div className="flex flex-col gap-4">
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.24em] text-orange-100/70">
            Workspace
          </p>
          <h2 className="mt-2 text-3xl font-semibold text-white">
            Connect the full `dustfril-core` workflow
          </h2>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-300">
            Scan artifact directories, inspect age and size, build a cleanup plan from safe
            candidates, and audit Node lifecycle scripts without leaving the desktop app.
          </p>
        </div>

        <label className="space-y-2">
          <span className="text-sm font-medium text-slate-200">Root path</span>
          <input
            value={props.root}
            onChange={(event) => props.onRootChange(event.currentTarget.value)}
            className="w-full rounded-2xl border border-white/10 bg-slate-950/45 px-4 py-3 text-sm text-white outline-none transition focus:border-orange-300/60"
            placeholder="/path/to/workspace"
          />
        </label>

        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-sm font-medium text-slate-200">Ecosystems</p>
            <div className="mt-3 flex flex-wrap gap-2">
              {ecosystems.map((ecosystem) => {
                const active = props.selectedEcosystems.includes(ecosystem);

                return (
                  <button
                    key={ecosystem}
                    type="button"
                    onClick={() => props.onToggleEcosystem(ecosystem)}
                    className={`rounded-full border px-4 py-2 text-sm transition ${
                      active
                        ? 'border-orange-300/50 bg-orange-300/15 text-orange-50'
                        : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/10'
                    }`}
                  >
                    {ecosystem}
                  </button>
                );
              })}
            </div>
          </div>

          <div>
            <p className="text-sm font-medium text-slate-200">Delete mode</p>
            <div className="mt-3 flex gap-2">
              {deleteModes.map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => props.onDeleteModeChange(mode)}
                  className={`rounded-full border px-4 py-2 text-sm transition ${
                    props.deleteMode === mode
                      ? 'border-cyan-300/50 bg-cyan-300/15 text-cyan-50'
                      : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/10'
                  }`}
                >
                  {mode}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <button
            type="button"
            onClick={props.onScan}
            disabled={disabled()}
            className="rounded-2xl bg-white px-4 py-3 text-sm font-medium text-slate-950 transition hover:bg-orange-100 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {props.busyAction === 'scan' ? 'Scanning...' : 'Run Scan'}
          </button>
          <button
            type="button"
            onClick={props.onAnalyze}
            disabled={disabled()}
            className="rounded-2xl border border-white/15 bg-white/6 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {props.busyAction === 'analyze' ? 'Analyzing...' : 'Analyze Artifacts'}
          </button>
          <button
            type="button"
            onClick={props.onBuildCleanupPlan}
            disabled={disabled()}
            className="rounded-2xl border border-white/15 bg-white/6 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {props.busyAction === 'cleanup-plan' ? 'Preparing...' : 'Build Cleanup Plan'}
          </button>
          <button
            type="button"
            onClick={props.onAudit}
            disabled={disabled()}
            className="rounded-2xl border border-white/15 bg-white/6 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {props.busyAction === 'audit' ? 'Auditing...' : 'Audit Scripts'}
          </button>
        </div>
      </div>
    </div>
  );
}
