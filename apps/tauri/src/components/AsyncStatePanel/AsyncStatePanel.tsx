import type { AsyncOperationStatus } from '../../model/async';

type AsyncStatePanelProps = {
  status: AsyncOperationStatus;
  title: string;
  description: string;
  warnings?: string[];
  error?: string;
  actionLabel?: string;
  onAction?: () => void;
};

const statusLabels: Record<AsyncOperationStatus, string> = {
  idle: 'Ready',
  loading: 'Loading',
  success: 'Complete',
  partial: 'Partial',
  unsupported: 'Planned',
  empty: 'No results',
  error: 'Error',
};

export function AsyncStatePanel(props: AsyncStatePanelProps) {
  const message = props.error ?? props.description;

  return (
    <section className={`async-state-panel async-state-${props.status}`} role="status">
      <div className="async-state-header">
        <span className="status-badge">{statusLabels[props.status]}</span>
        <h2>{props.title}</h2>
      </div>
      <p>{message}</p>
      {props.warnings?.length ? (
        <ul className="async-state-warnings">
          {props.warnings.map((warning) => (
            <li key={warning}>{warning}</li>
          ))}
        </ul>
      ) : null}
      {props.actionLabel && props.onAction ? (
        <button type="button" className="button-secondary" onClick={props.onAction}>
          {props.actionLabel}
        </button>
      ) : null}
    </section>
  );
}
