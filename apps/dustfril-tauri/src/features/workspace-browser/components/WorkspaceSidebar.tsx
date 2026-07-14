import type { DeleteMode, Ecosystem } from '../../../types/workflow';
import type { PaneConfig, StatusMetric, BrowserPane } from '../model/types';
import { SidebarActionButton, StatusRow } from './shared';

type WorkspaceSidebarProps = {
  paneConfigs: PaneConfig[];
  activePane: BrowserPane;
  selectedEcosystems: Ecosystem[];
  deleteMode: DeleteMode;
  statusMetrics: StatusMetric[];
  busyAction: string | null;
  canRunActions: boolean;
  onPaneChange: (pane: BrowserPane) => void;
  onToggleEcosystem: (ecosystem: Ecosystem) => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  onScan: () => void | Promise<void>;
  onAnalyze: () => void | Promise<void>;
  onBuildCleanupPlan: () => void | Promise<void>;
  onAudit: () => void | Promise<void>;
  ecosystems: Ecosystem[];
  deleteModes: DeleteMode[];
};

export function WorkspaceSidebar(props: WorkspaceSidebarProps) {
  return (
    <aside className="border-r border-white/8 bg-[linear-gradient(180deg,#242426,#1d1d20)] px-4 py-4">
      <div className="space-y-5">
        <div>
          <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">Views</p>
          <div className="space-y-1.5">
            {props.paneConfigs.map((pane) => {
              const active = pane.key === props.activePane;

              return (
                <button
                  key={pane.key}
                  type="button"
                  onClick={() => props.onPaneChange(pane.key)}
                  className={`flex w-full items-center justify-between rounded-2xl px-3 py-3 text-left transition ${
                    active
                      ? 'bg-[#3a3a3c] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
                      : 'text-slate-300 hover:bg-white/6'
                  }`}
                >
                  <div className="min-w-0">
                    <p className={`truncate text-sm font-medium ${pane.accent}`}>{pane.title}</p>
                    <p className="mt-1 truncate text-xs text-slate-500">{pane.description}</p>
                  </div>
                  <span className="ml-3 rounded-full bg-black/20 px-2.5 py-1 text-xs text-slate-300">
                    {pane.count}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        <div>
          <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
            Ecosystems
          </p>
          <div className="flex flex-wrap gap-2">
            {props.ecosystems.map((ecosystem) => {
              const active = props.selectedEcosystems.includes(ecosystem);

              return (
                <button
                  key={ecosystem}
                  type="button"
                  onClick={() => props.onToggleEcosystem(ecosystem)}
                  className={`rounded-full border px-3 py-1.5 text-xs transition ${
                    active
                      ? 'border-sky-300/35 bg-sky-400/12 text-sky-50'
                      : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/8'
                  }`}
                >
                  {ecosystem}
                </button>
              );
            })}
          </div>
        </div>

        <div>
          <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">Actions</p>
          <div className="grid gap-2">
            <SidebarActionButton
              label={props.busyAction === 'scan' ? 'Scanning...' : 'Run Scan'}
              onClick={props.onScan}
              disabled={!props.canRunActions}
            />
            <SidebarActionButton
              label={props.busyAction === 'analyze' ? 'Analyzing...' : 'Analyze'}
              onClick={props.onAnalyze}
              disabled={!props.canRunActions}
            />
            <SidebarActionButton
              label={props.busyAction === 'cleanup-plan' ? 'Preparing...' : 'Build Cleanup'}
              onClick={props.onBuildCleanupPlan}
              disabled={!props.canRunActions}
            />
            <SidebarActionButton
              label={props.busyAction === 'audit' ? 'Auditing...' : 'Audit Scripts'}
              onClick={props.onAudit}
              disabled={!props.canRunActions}
            />
          </div>
        </div>

        <div>
          <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
            Delete Mode
          </p>
          <div className="grid grid-cols-2 gap-2">
            {props.deleteModes.map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => props.onDeleteModeChange(mode)}
                className={`rounded-2xl px-3 py-2 text-sm transition ${
                  props.deleteMode === mode
                    ? 'bg-[#3a3a3c] text-white'
                    : 'bg-black/15 text-slate-300 hover:bg-white/8'
                }`}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>

        <div className="rounded-[22px] border border-white/8 bg-black/15 p-4">
          <p className="text-xs uppercase tracking-[0.2em] text-slate-500">Status</p>
          <div className="mt-3 grid gap-2 text-sm text-slate-300">
            {props.statusMetrics.map((metric) => (
              <StatusRow key={metric.label} label={metric.label} value={metric.value} />
            ))}
          </div>
        </div>
      </div>
    </aside>
  );
}
