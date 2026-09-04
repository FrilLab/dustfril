type ClearHistoryDialogProps = {
  open: boolean;
  busy: boolean;
  error?: string | null;
  onCancel: () => void;
  onConfirm: () => void | Promise<void>;
};

export function ClearHistoryDialog(props: ClearHistoryDialogProps) {
  if (!props.open) {
    return null;
  }

  return (
    <div className="dialog-backdrop">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="clear-history-dialog-title"
        className="cleanup-dialog clear-history-dialog"
      >
        <p className="eyebrow">Activity History</p>
        <h2 id="clear-history-dialog-title">Clear Activity History?</h2>
        <p className="dialog-copy">
          This removes DustFril&apos;s local scan, cleanup, and activity history. It does not delete
          project files or cleanup artifacts.
        </p>
        {props.error ? (
          <p className="dialog-error" role="alert">
            {props.error}
          </p>
        ) : null}

        <div className="dialog-actions">
          <button
            type="button"
            onClick={props.onCancel}
            disabled={props.busy}
            className="button-secondary"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void props.onConfirm()}
            disabled={props.busy}
            className="button-confirm button-confirm-danger"
          >
            {props.busy ? 'Clearing…' : 'Clear History'}
          </button>
        </div>
      </div>
    </div>
  );
}
