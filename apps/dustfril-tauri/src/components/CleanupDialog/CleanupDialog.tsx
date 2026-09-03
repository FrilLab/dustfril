import { formatBytes } from '../../lib/format';
import type { CleanupCandidate, DeleteMode, Recommendation } from '../../types/workflow';

type CleanupDialogProps = {
  open: boolean;
  itemCount: number;
  totalBytes: number;
  deleteMode: DeleteMode;
  selectedItems: CleanupCandidate[];
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void | Promise<void>;
};

export function CleanupDialog(props: CleanupDialogProps) {
  if (!props.open) {
    return null;
  }

  const isTrash = props.deleteMode === 'Trash';
  const actionLabel = isTrash ? 'Move to Trash' : 'Delete Permanently';
  const recommendedItems = props.selectedItems.filter(
    (item) => item.recommendation === 'SafeToClean',
  );
  const manuallySelectedItems = props.selectedItems.filter(
    (item) => item.recommendation !== 'SafeToClean',
  );

  return (
    <div className="dialog-backdrop">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="cleanup-dialog-title"
        className="cleanup-dialog"
      >
        <p className="eyebrow">Review Cleanup</p>
        <h2 id="cleanup-dialog-title">{actionLabel}?</h2>
        <p className="dialog-copy">
          {isTrash
            ? 'The selected artifacts will be moved to the system Trash, where they can still be recovered.'
            : 'This permanently deletes the selected artifacts. This action cannot be undone.'}
        </p>

        <div className="dialog-summary">
          <span>{props.itemCount} selected</span>
          <strong>{formatBytes(props.totalBytes)}</strong>
        </div>

        {props.selectedItems.length ? (
          <div className="dialog-paths">
            {recommendedItems.length ? (
              <ReviewGroup title="Recommended" items={recommendedItems} />
            ) : null}
            {manuallySelectedItems.length ? (
              <ReviewGroup title="Manually selected" items={manuallySelectedItems} />
            ) : null}
          </div>
        ) : null}

        <div className="dialog-actions">
          <button type="button" onClick={props.onCancel} disabled={props.busy} className="button-secondary">
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void props.onConfirm()}
            disabled={props.busy}
            className={`button-confirm${isTrash ? '' : ' button-confirm-danger'}`}
          >
            {props.busy ? 'Cleaning…' : actionLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

function ReviewGroup({ title, items }: { title: string; items: CleanupCandidate[] }) {
  return (
    <div className="dialog-review-group">
      <p className="eyebrow">{title}</p>
      <ul>
        {items.slice(0, 5).map((item) => (
          <li key={item.path} title={item.path}>
            <span>{item.path}</span>
            {item.recommendation !== 'SafeToClean' ? (
              <small>{recommendationLabel(item.recommendation)}</small>
            ) : null}
          </li>
        ))}
        {items.length > 5 ? <li className="dialog-more">+ {items.length - 5} more</li> : null}
      </ul>
    </div>
  );
}

function recommendationLabel(recommendation: Recommendation) {
  switch (recommendation) {
    case 'NeedsReview':
      return 'Needs review';
    case 'Keep':
      return 'Keep';
    case 'SafeToClean':
      return 'Recommended';
  }
}
