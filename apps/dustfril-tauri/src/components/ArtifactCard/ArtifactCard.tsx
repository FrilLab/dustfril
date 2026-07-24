import type { BrowserItem } from '../../features/workspace-browser/model/types';
import { ItemIcon } from '../../features/workspace-browser/components/icons';

type ArtifactCardProps = {
  item: BrowserItem;
  selected: boolean;
  selectable?: boolean;
  checked?: boolean;
  onSelect: (itemId: string) => void;
  onToggleCheck?: (path: string) => void;
};

export function ArtifactCard(props: ArtifactCardProps) {
  return (
    <div
      className={`flex w-full items-center gap-3 px-4 py-3 transition ${
        props.selected ? 'bg-[#4a4a4f]/70' : 'hover:bg-white/5'
      }`}
    >
      {props.selectable && props.item.path ? (
        <input
          type="checkbox"
          checked={props.checked ?? false}
          onChange={() => props.onToggleCheck?.(props.item.path!)}
          className="h-4 w-4 rounded border-white/20 bg-black/20 accent-cyan-400"
        />
      ) : null}
      <button
        type="button"
        onClick={() => props.onSelect(props.item.id)}
        className="flex min-w-0 flex-1 items-center gap-3 text-left"
      >
        <ItemIcon kind={props.item.kind} />
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium text-white">{props.item.title}</p>
          <p className="mt-0.5 truncate text-xs text-slate-400">{props.item.subtitle}</p>
        </div>
        <div className="hidden text-right md:block">
          <p className="text-xs text-slate-300">{props.item.meta}</p>
          <span
            className={`mt-1 inline-flex rounded-full border px-2 py-0.5 text-[10px] ${props.item.accent}`}
          >
            {props.item.badge}
          </span>
        </div>
      </button>
    </div>
  );
}
