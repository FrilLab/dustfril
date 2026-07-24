import { ArtifactList } from '../../../components/ArtifactList/ArtifactList';
import { StorageSummary } from '../../../components/StorageSummary/StorageSummary';
import { EmptyState } from '../../../components/EmptyState/EmptyState';
import type { BrowserItem } from '../../workspace-browser/model/types';
import { formatBytes, formatCount } from '../../../lib/format';
import type { CategoryConfig } from '../../../model/categories';
import type { DeleteMode } from '../../../types/workflow';

type CategoryCleanupViewProps = {
  category: CategoryConfig;
  scanItems: BrowserItem[];
  analysisItems: BrowserItem[];
  cleanupItems: BrowserItem[];
  selectedItemId: string | null;
  selectedCleanupPaths: string[];
  selectedCandidateBytes: number;
  deleteMode: DeleteMode;
  busyAction: string | null;
  canRunActions: boolean;
  onSelectItem: (itemId: string) => void;
  onToggleCleanupPath: (path: string) => void;
  onScanCategory: () => void | Promise<void>;
  onBuildCleanupPlan: () => void | Promise<void>;
  onRequestCleanup: () => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  deleteModes: DeleteMode[];
};

export function CategoryCleanupView(props: CategoryCleanupViewProps) {
  const selectedItem =
    [...props.scanItems, ...props.analysisItems, ...props.cleanupItems].find(
      (item) => item.id === props.selectedItemId,
    ) ?? null;

  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Category Cleanup</p>
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

      <div className="grid gap-4 xl:grid-cols-2">
        <ArtifactList
          title="Scan Results"
          description="Discovered artifact paths"
          items={props.scanItems}
          selectedItemId={props.selectedItemId}
          emptyMessage="No scan results yet. Run Scan to discover artifacts in this category."
          onSelectItem={props.onSelectItem}
        />
        <ArtifactList
          title="Analysis"
          description="Size, age, and cleanup recommendation"
          items={props.analysisItems}
          selectedItemId={props.selectedItemId}
          emptyMessage="No analysis data yet. Scan the workspace to populate this view."
          onSelectItem={props.onSelectItem}
        />
      </div>

      <ArtifactList
        title="Cleanup Queue"
        description="Select items before confirming cleanup"
        items={props.cleanupItems}
        selectedItemId={props.selectedItemId}
        selectable
        selectedPaths={props.selectedCleanupPaths}
        emptyMessage="No cleanup candidates yet. Build a cleanup plan after scanning."
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

      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Selected Item</p>
        {selectedItem ? (
          <div className="mt-3 space-y-2 text-sm text-slate-300">
            <p className="font-medium text-white">{selectedItem.title}</p>
            <p className="break-all text-slate-400">{selectedItem.subtitle}</p>
            {selectedItem.detailLines.map((line) => (
              <p key={line}>{line}</p>
            ))}
          </div>
        ) : (
          <EmptyState message="Select an artifact to inspect path, size, and recommendation details." compact />
        )}
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
