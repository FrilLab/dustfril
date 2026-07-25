import type { BrowserItem } from '../../model/types';
import { ArtifactCard } from '../ArtifactCard/ArtifactCard';
import { EmptyState } from '../EmptyState/EmptyState';

type ArtifactListProps = {
  title: string;
  description: string;
  items: BrowserItem[];
  selectedItemId: string | null;
  emptyMessage: string;
  selectable?: boolean;
  selectedPaths?: string[];
  onSelectItem: (itemId: string) => void;
  onTogglePath?: (path: string) => void;
};

export function ArtifactList(props: ArtifactListProps) {
  return (
    <section className="min-h-0 overflow-hidden rounded-[24px] border border-white/8 bg-black/10">
      <div className="border-b border-white/8 bg-[#2b2b2e] px-4 py-3">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">{props.title}</p>
        <h2 className="mt-1 text-lg font-semibold text-white">{props.description}</h2>
      </div>

      <div className="min-h-[320px] overflow-y-auto">
        {props.items.length ? (
          <div className="divide-y divide-white/6">
            {props.items.map((item) => (
              <ArtifactCard
                key={item.id}
                item={item}
                selected={item.id === props.selectedItemId}
                selectable={props.selectable}
                checked={item.path ? props.selectedPaths?.includes(item.path) : false}
                onSelect={props.onSelectItem}
                onToggleCheck={props.onTogglePath}
              />
            ))}
          </div>
        ) : (
          <EmptyState message={props.emptyMessage} />
        )}
      </div>
    </section>
  );
}
