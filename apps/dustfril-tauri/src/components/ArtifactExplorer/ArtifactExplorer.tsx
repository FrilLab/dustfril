import { formatCount } from '../../lib/format';
import type { BrowserItem, ExplorerWorkflow } from '../../model/types';
import { EmptyState } from '../EmptyState/EmptyState';
import { ItemIcon } from '../icons';

const workflowLabels: Record<ExplorerWorkflow, { title: string; description: string }> = {
  scan: {
    title: 'Scan Results',
    description: 'Discovered artifact paths',
  },
  analysis: {
    title: 'Analysis',
    description: 'Size, age, and cleanup recommendation',
  },
  cleanup: {
    title: 'Cleanup Queue',
    description: 'Select items before confirming cleanup',
  },
};

type ArtifactExplorerProps = {
  workflow: ExplorerWorkflow;
  items: BrowserItem[];
  selectedItemId: string | null;
  emptyMessage: string;
  selectable?: boolean;
  selectedPaths?: string[];
  onSelectItem: (itemId: string) => void;
  onTogglePath?: (path: string) => void;
};

export function ArtifactExplorer(props: ArtifactExplorerProps) {
  const selectedItem = props.items.find((item) => item.id === props.selectedItemId) ?? null;
  const labels = workflowLabels[props.workflow];

  return (
    <section className="min-h-[520px] overflow-hidden rounded-[24px] border border-white/8 bg-black/10">
      <div className="border-b border-white/8 bg-[#2b2b2e] px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">{labels.title}</p>
            <h2 className="mt-1 text-lg font-semibold text-white">{labels.description}</h2>
          </div>
          <div className="rounded-full bg-black/20 px-3 py-1 text-xs text-slate-300">
            {formatCount(props.items.length)} visible
          </div>
        </div>
      </div>

      <div className="grid min-h-0 xl:grid-cols-[minmax(0,1.2fr)_240px_320px]">
        <div className="min-h-[420px] overflow-y-auto border-r border-white/8">
          {props.items.length ? (
            <div className="divide-y divide-white/6">
              {props.items.map((item) => {
                const selected = item.id === props.selectedItemId;

                return (
                  <div
                    key={item.id}
                    className={`flex items-center gap-3 px-4 py-3 transition ${
                      selected ? 'bg-[#4a4a4f]/70' : 'hover:bg-white/5'
                    }`}
                  >
                    {props.selectable && item.path ? (
                      <input
                        type="checkbox"
                        checked={props.selectedPaths?.includes(item.path) ?? false}
                        onChange={() => props.onTogglePath?.(item.path!)}
                        className="h-4 w-4 rounded border-white/20 bg-black/20 accent-cyan-400"
                      />
                    ) : null}
                    <button
                      type="button"
                      onClick={() => props.onSelectItem(item.id)}
                      className="flex min-w-0 flex-1 items-center gap-3 text-left"
                    >
                      <ItemIcon kind={item.kind} />
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-medium text-white">{item.title}</p>
                        <p className="mt-0.5 truncate text-xs text-slate-400">{item.subtitle}</p>
                      </div>
                      <p className="hidden text-xs text-slate-300 md:block">{item.meta}</p>
                    </button>
                  </div>
                );
              })}
            </div>
          ) : (
            <EmptyState message={props.emptyMessage} />
          )}
        </div>

        <div className="min-h-[420px] border-r border-white/8 bg-[#202023]">
          <div className="border-b border-white/8 px-4 py-3">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Preview</p>
          </div>
          <div className="overflow-y-auto px-4 py-4">
            <ExplorerPreview item={selectedItem} />
          </div>
        </div>

        <div className="min-h-[420px] bg-[linear-gradient(180deg,#222225,#1b1b1d)]">
          <div className="border-b border-white/8 px-4 py-3">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Inspector</p>
          </div>
          <div className="overflow-y-auto px-4 py-4">
            <ExplorerPreview item={selectedItem} detailed />
          </div>
        </div>
      </div>
    </section>
  );
}

function ExplorerPreview(props: { item: BrowserItem | null; detailed?: boolean }) {
  if (!props.item) {
    return (
      <EmptyState
        message="Select an artifact to inspect path, size, and recommendation details."
        compact
      />
    );
  }

  return (
    <div>
      <div className="flex items-start gap-3">
        <ItemIcon kind={props.item.kind} large={props.detailed} />
        <div className="min-w-0">
          <p className="break-all text-sm font-semibold text-white">{props.item.title}</p>
          <p className="mt-1 break-all text-xs text-slate-400">{props.item.subtitle}</p>
        </div>
      </div>
      <div className={`mt-4 inline-flex rounded-full border px-3 py-1 text-xs ${props.item.accent}`}>
        {props.item.badge}
      </div>
      <div className="mt-4 space-y-2 text-sm text-slate-300">
        {props.item.detailLines.map((line) => (
          <p key={line}>{line}</p>
        ))}
      </div>
    </div>
  );
}
