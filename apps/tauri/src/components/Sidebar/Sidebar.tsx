import {
  categorySections,
  type CategoryConfig,
  type CategorySection,
  type SidebarCategory,
} from '../../model/categories';

export type SidebarEntry = CategoryConfig & {
  count: number | null;
};

type SidebarProps = {
  entries: SidebarEntry[];
  activeCategory: SidebarCategory;
  onCategoryChange: (category: SidebarCategory) => void;
};

export function Sidebar(props: SidebarProps) {
  const entriesBySection = new Map<CategorySection, SidebarEntry[]>();
  for (const entry of props.entries) {
    const sectionEntries = entriesBySection.get(entry.section) ?? [];
    sectionEntries.push(entry);
    entriesBySection.set(entry.section, sectionEntries);
  }

  return (
    <aside className="app-sidebar">
      <nav className="sidebar-nav" aria-label="Primary navigation">
        {categorySections.map((section) => {
          const entries = entriesBySection.get(section.key) ?? [];
          if (!entries.length) {
            return null;
          }

          return (
            <div className="sidebar-section" key={section.key}>
              <p className="sidebar-heading">{section.title}</p>
              {entries.map((entry) => (
                <SidebarButton
                  key={entry.key}
                  entry={entry}
                  active={isCategoryActive(entry.key, props.activeCategory)}
                  onCategoryChange={props.onCategoryChange}
                />
              ))}
            </div>
          );
        })}
      </nav>
    </aside>
  );
}

function isCategoryActive(entry: SidebarCategory, activeCategory: SidebarCategory) {
  if (entry === 'workspace') {
    return activeCategory === 'workspace' || activeCategory.startsWith('cleanup-');
  }
  if (entry === 'history') {
    return activeCategory === 'history' || activeCategory === 'workspace-activity';
  }
  return entry === activeCategory;
}

function SidebarButton({
  entry,
  active,
  onCategoryChange,
}: {
  entry: SidebarEntry;
  active: boolean;
  onCategoryChange: (category: SidebarCategory) => void;
}) {
  const planned = entry.availability === 'planned';

  return (
    <button
      type="button"
      onClick={() => onCategoryChange(entry.key)}
      className={`sidebar-item${active ? ' sidebar-item-active' : ''}${planned ? ' sidebar-item-planned' : ''}`}
      aria-current={active ? 'page' : undefined}
      title={entry.description}
    >
      <span className={`sidebar-item-icon sidebar-icon-${entry.section}`} aria-hidden="true">
        {entry.section === 'favorites'
          ? '◌'
          : entry.section === 'cleanup'
            ? '▣'
            : entry.section === 'workspace'
              ? '◷'
              : '◇'}
      </span>
      <span className="sidebar-item-label">{entry.title}</span>
      {planned ? <span className="sidebar-planned-label" aria-hidden="true">Planned</span> : null}
      {entry.count !== null ? <span className="sidebar-count" aria-hidden="true">{entry.count}</span> : null}
    </button>
  );
}
