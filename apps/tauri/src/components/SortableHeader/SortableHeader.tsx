import type { SortDirection } from '../../model/sorting';

type SortableHeaderProps = {
  label: string;
  active: boolean;
  direction: SortDirection;
  onSort: () => void;
};

export function SortableHeader(props: SortableHeaderProps) {
  return (
    <div
      role="columnheader"
      aria-sort={props.active ? (props.direction === 'asc' ? 'ascending' : 'descending') : 'none'}
    >
      <button
        type="button"
        className="sortable-header"
        aria-label={props.label}
        title={`Sort by ${props.label}`}
        onClick={props.onSort}
      >
        <span>{props.label}</span>
        {props.active ? (
          <span className="sort-indicator" aria-hidden="true">
            {props.direction === 'asc' ? '↑' : '↓'}
          </span>
        ) : null}
      </button>
    </div>
  );
}
