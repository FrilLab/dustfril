import type { CategoryConfig, SidebarCategory } from '../../model/categories';

export type SidebarEntry = CategoryConfig & {
  count: number | null;
};

type SidebarProps = {
  entries: SidebarEntry[];
  activeCategory: SidebarCategory;
  onCategoryChange: (category: SidebarCategory) => void;
};

export function Sidebar(props: SidebarProps) {
  return (
    <aside className="app-sidebar">
      <p className="sidebar-heading">Favorites</p>
      <nav className="sidebar-nav" aria-label="Primary navigation">
        {props.entries.map((entry) => {
          const active = entry.key === props.activeCategory;

          return (
            <button
              key={entry.key}
              type="button"
              onClick={() => props.onCategoryChange(entry.key)}
              className={`sidebar-item${active ? ' sidebar-item-active' : ''}`}
              aria-current={active ? 'page' : undefined}
              title={entry.description}
            >
              <span className={`sidebar-item-icon sidebar-icon-${entry.key}`} aria-hidden="true">
                {entry.key === 'overview' ? '◌' : entry.key === 'workspace' ? '▣' : '◷'}
              </span>
              <span className="sidebar-item-label">{entry.title}</span>
              {entry.count !== null ? <span className="sidebar-count">{entry.count}</span> : null}
            </button>
          );
        })}
      </nav>

    </aside>
  );
}
