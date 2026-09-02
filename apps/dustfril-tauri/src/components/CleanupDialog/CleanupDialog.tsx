import { formatBytes } from '../../lib/format';
import type { DeleteMode } from '../../types/workflow';

type CleanupDialogProps = {
  open: boolean;
  itemCount: number;
  totalBytes: number;
  deleteMode: DeleteMode;
  samplePaths: string[];
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

        {props.samplePaths.length ? (
          <div className="dialog-paths">
            <p className="eyebrow">Selected paths</p>
            <ul>
              {props.samplePaths.map((path) => (
                <li key={path} title={path}>
                  {path}
                </li>
              ))}
              {props.itemCount > props.samplePaths.length ? (
                <li className="dialog-more">+ {props.itemCount - props.samplePaths.length} more</li>
              ) : null}
            </ul>
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
