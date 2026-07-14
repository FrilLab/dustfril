import { formatCount } from '../../../lib/format';
import type { BrowserItem, PaneConfig } from '../model/types';
import { ItemIcon } from './icons';
import { EmptyState } from './shared';

type BrowserPaneProps = {
  activePaneConfig: PaneConfig;
  items: BrowserItem[];
  selectedItemId: string | null;
  onSelectItem: (itemId: string) => void;
};

export function BrowserPane(props: BrowserPaneProps) {
  return (
    <section className="min-h-0 border-r border-white/8">
      <div className="border-b border-white/8 bg-[#2b2b2e] px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Current View</p>
            <h2 className="mt-1 text-lg font-semibold text-white">{props.activePaneConfig.title}</h2>
          </div>
          <div className="rounded-full bg-black/20 px-3 py-1 text-xs text-slate-300">
            {formatCount(props.items.length)} visible
          </div>
        </div>
      </div>

      <div className="grid min-h-0 md:grid-cols-[minmax(0,1fr)_220px]">
        <div className="min-h-0 overflow-y-auto">
          {props.items.length ? (
            <div className="divide-y divide-white/6">
              {props.items.map((item) => {
                const selected = item.id === props.selectedItemId;

                return (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => props.onSelectItem(item.id)}
                    className={`flex w-full items-center gap-3 px-4 py-3 text-left transition ${
                      selected ? 'bg-[#4a4a4f]/70' : 'hover:bg-white/5'
                    }`}
                  >
                    <ItemIcon kind={item.kind} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-white">{item.title}</p>
                      <p className="mt-0.5 truncate text-xs text-slate-400">{item.subtitle}</p>
                    </div>
                    <div className="hidden text-right md:block">
                      <p className="text-xs text-slate-300">{item.meta}</p>
                    </div>
                  </button>
                );
              })}
            </div>
          ) : (
            <EmptyState message="No items in this view. Run the action again or broaden the filter." />
          )}
        </div>

        <div className="min-h-0 border-t border-white/8 bg-[#202023] md:border-l md:border-t-0">
          <div className="border-b border-white/8 px-4 py-3">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Preview</p>
          </div>
          <div className="overflow-y-auto px-4 py-4">
            <PanePreview item={props.items.find((item) => item.id === props.selectedItemId) ?? null} />
          </div>
        </div>
      </div>
    </section>
  );
}

function PanePreview(props: { item: BrowserItem | null }) {
  if (!props.item) {
    return <EmptyState message="Select an item to inspect details." compact />;
  }

  return (
    <div>
      <div className="flex items-start gap-3">
        <ItemIcon kind={props.item.kind} large />
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
