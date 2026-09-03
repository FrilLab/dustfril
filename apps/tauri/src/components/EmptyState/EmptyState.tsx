import type { ReactNode } from 'react';

type EmptyStateProps = {
  message: string;
  compact?: boolean;
  icon?: ReactNode;
};

export function EmptyState(props: EmptyStateProps) {
  return (
    <div className={`empty-state${props.compact ? ' empty-state-compact' : ''}`}>
      {props.icon ? <div className="empty-state-icon">{props.icon}</div> : null}
      <p>{props.message}</p>
    </div>
  );
}
