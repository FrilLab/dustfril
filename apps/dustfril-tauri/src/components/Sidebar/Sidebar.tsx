import type { CategoryConfig, SidebarCategory } from '../../model/categories';

export type SidebarEntry = CategoryConfig & {
  count: number;
};

type SidebarProps = {
  entries: SidebarEntry[];
  activeCategory: SidebarCategory;
  onCategoryChange: (category: SidebarCategory) => void;
};

export function Sidebar(props: SidebarProps) {
  const primaryEntries = props.entries.filter((entry) => entry.section === 'primary');
  const languageEntries = props.entries.filter((entry) => entry.section === 'language');
  const futureEntries = props.entries.filter((entry) => entry.section === 'future');

  return (
    <aside className="border-r border-white/8 bg-[linear-gradient(180deg,#242426,#1d1d20)] px-4 py-4">
      <div className="space-y-5">
        <SidebarSection
          title="Overview"
          entries={primaryEntries}
          activeCategory={props.activeCategory}
          onCategoryChange={props.onCategoryChange}
        />
        <SidebarSection
          title="Languages"
          entries={languageEntries}
          activeCategory={props.activeCategory}
          onCategoryChange={props.onCategoryChange}
        />
        <SidebarSection
          title="Future"
          entries={futureEntries}
          activeCategory={props.activeCategory}
          onCategoryChange={props.onCategoryChange}
        />
      </div>
    </aside>
  );
}

function SidebarSection(props: {
  title: string;
  entries: SidebarEntry[];
  activeCategory: SidebarCategory;
  onCategoryChange: (category: SidebarCategory) => void;
}) {
  return (
    <div>
      <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
        {props.title}
      </p>
      <div className="space-y-1.5">
        {props.entries.map((entry) => {
          const active = entry.key === props.activeCategory;

          return (
            <button
              key={entry.key}
              type="button"
              onClick={() => props.onCategoryChange(entry.key)}
              className={`flex w-full items-center justify-between rounded-2xl px-3 py-3 text-left transition ${
                active
                  ? 'bg-[#3a3a3c] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
                  : 'text-slate-300 hover:bg-white/6'
              }`}
            >
              <div className="min-w-0">
                <p className="truncate text-sm font-medium text-white">{entry.title}</p>
                <p className="mt-1 truncate text-xs text-slate-500">{entry.description}</p>
              </div>
              <span className="ml-3 rounded-full bg-black/20 px-2.5 py-1 text-xs text-slate-300">
                {entry.count}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
