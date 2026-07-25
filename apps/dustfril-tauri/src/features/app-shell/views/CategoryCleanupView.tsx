import { ArtifactExplorer } from '../../../components/ArtifactExplorer/ArtifactExplorer';
import { StorageSummary } from '../../../components/StorageSummary/StorageSummary';
import { formatBytes, formatCount } from '../../../lib/format';
import type { CategoryConfig } from '../../../model/categories';
import type { ExplorerWorkflow } from '../../../model/types';
import type { BrowserItem } from '../../../model/types';
import type { DeleteMode } from '../../../types/workflow';

type CategoryCleanupViewProps = {
  category: CategoryConfig;
  explorerWorkflow: ExplorerWorkflow;
  explorerItems: BrowserItem[];
  scanItems: BrowserItem[];
  analysisItems: BrowserItem[];
  cleanupItems: BrowserItem[];
  selectedItemId: string | null;
  selectedCleanupPaths: string[];
  selectedCandidateBytes: number;
  deleteMode: DeleteMode;
  busyAction: string | null;
  canRunActions: boolean;
  onWorkflowChange: (workflow: ExplorerWorkflow) => void;
  onSelectItem: (itemId: string) => void;
  onToggleCleanupPath: (path: string) => void;
  onScanCategory: () => void | Promise<void>;
  onAnalyzeCategory: () => void | Promise<void>;
  onBuildCleanupPlan: () => void | Promise<void>;
  onRequestCleanup: () => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  deleteModes: DeleteMode[];
};

export function CategoryCleanupView(props: CategoryCleanupViewProps) {
  const emptyMessages: Record<ExplorerWorkflow, string> = {
    scan: 'No scan results yet. Run Scan to discover artifacts in this category.',
    analysis: 'No analysis data yet. Analyze the workspace to populate this view.',
    cleanup: 'No cleanup candidates yet. Build a cleanup plan after scanning.',
  };

  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Artifact Explorer</p>
            <h2 className="mt-1 text-2xl font-semibold text-white">{props.category.title}</h2>
            <p className="mt-2 text-sm text-slate-300">{props.category.description}</p>
          </div>
          <div className="flex flex-wrap gap-2">
            <ActionButton
              label={props.busyAction === 'scan' ? 'Scanning...' : 'Scan'}
              onClick={props.onScanCategory}
              disabled={!props.canRunActions}
            />
            <ActionButton
              label={props.busyAction === 'analyze' ? 'Analyzing...' : 'Analyze'}
              onClick={props.onAnalyzeCategory}
              disabled={!props.canRunActions}
            />
            <ActionButton
              label={props.busyAction === 'cleanup-plan' ? 'Preparing...' : 'Build Cleanup Plan'}
              onClick={props.onBuildCleanupPlan}
              disabled={!props.canRunActions}
            />
            <ActionButton
              label={`Review Cleanup (${formatCount(props.selectedCleanupPaths.length)})`}
              onClick={props.onRequestCleanup}
              disabled={props.selectedCleanupPaths.length === 0 || props.busyAction !== null}
              primary
            />
          </div>
        </div>
      </section>

      <StorageSummary
        title="Category Metrics"
        metrics={[
          { label: 'Scan Results', value: formatCount(props.scanItems.length) },
          { label: 'Analyzed Artifacts', value: formatCount(props.analysisItems.length) },
          { label: 'Cleanup Candidates', value: formatCount(props.cleanupItems.length) },
          {
            label: 'Selected For Cleanup',
            value: `${formatCount(props.selectedCleanupPaths.length)} · ${formatBytes(props.selectedCandidateBytes)}`,
          },
        ]}
      />

      <div className="flex flex-wrap gap-2">
        {(['scan', 'analysis', 'cleanup'] as ExplorerWorkflow[]).map((workflow) => (
          <button
            key={workflow}
            type="button"
            onClick={() => props.onWorkflowChange(workflow)}
            className={`rounded-full px-4 py-2 text-sm transition ${
              props.explorerWorkflow === workflow
                ? 'bg-[#3a3a3c] text-white'
                : 'border border-white/10 bg-white/6 text-slate-300 hover:bg-white/10'
            }`}
          >
            {workflow === 'scan' ? 'Scan' : workflow === 'analysis' ? 'Analyze' : 'Cleanup'}
          </button>
        ))}
      </div>

      <ArtifactExplorer
        workflow={props.explorerWorkflow}
        items={props.explorerItems}
        selectedItemId={props.selectedItemId}
        selectable={props.explorerWorkflow === 'cleanup'}
        selectedPaths={props.selectedCleanupPaths}
        emptyMessage={emptyMessages[props.explorerWorkflow]}
        onSelectItem={props.onSelectItem}
        onTogglePath={props.onToggleCleanupPath}
      />

      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Delete Mode</p>
        <div className="mt-3 grid max-w-md grid-cols-2 gap-2">
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
      </section>
    </div>
  );
}

function ActionButton(props: {
  label: string;
  onClick: () => void | Promise<void>;
  disabled?: boolean;
  primary?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={() => void props.onClick()}
      disabled={props.disabled}
      className={`rounded-2xl px-4 py-3 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-50 ${
        props.primary
          ? 'border border-cyan-300/20 bg-cyan-400/10 text-cyan-50 hover:bg-cyan-400/18'
          : 'border border-white/10 bg-white/6 text-white hover:bg-white/10'
      }`}
    >
      {props.label}
    </button>
  );
}
