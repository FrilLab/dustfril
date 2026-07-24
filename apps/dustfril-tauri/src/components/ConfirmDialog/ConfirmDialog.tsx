import { formatBytes } from '../../lib/format';
import type { DeleteMode } from '../../types/workflow';

type ConfirmDialogProps = {
  open: boolean;
  itemCount: number;
  totalBytes: number;
  deleteMode: DeleteMode;
  samplePaths: string[];
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void | Promise<void>;
};

export function ConfirmDialog(props: ConfirmDialogProps) {
  if (!props.open) {
    return null;
  }

  const actionLabel = props.deleteMode === 'Trash' ? 'Move to Trash' : 'Delete Permanently';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4 backdrop-blur-sm">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        className="w-full max-w-lg rounded-[28px] border border-white/10 bg-[#242426] p-6 shadow-[0_30px_80px_rgba(0,0,0,0.45)]"
      >
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Confirm Cleanup</p>
        <h2 id="confirm-dialog-title" className="mt-2 text-xl font-semibold text-white">
          {actionLabel}?
        </h2>
        <p className="mt-3 text-sm leading-6 text-slate-300">
          You are about to clean {props.itemCount} item(s) totaling {formatBytes(props.totalBytes)}.
          Review the selected paths before continuing.
        </p>

        {props.samplePaths.length ? (
          <div className="mt-4 rounded-2xl border border-white/8 bg-black/20 p-4">
            <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Selected paths</p>
            <ul className="mt-2 space-y-1 text-sm text-slate-300">
              {props.samplePaths.map((path) => (
                <li key={path} className="truncate">
                  {path}
                </li>
              ))}
              {props.itemCount > props.samplePaths.length ? (
                <li className="text-slate-500">
                  + {props.itemCount - props.samplePaths.length} more
                </li>
              ) : null}
            </ul>
          </div>
        ) : null}

        <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <button
            type="button"
            onClick={props.onCancel}
            disabled={props.busy}
            className="rounded-2xl border border-white/10 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/8 disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void props.onConfirm()}
            disabled={props.busy}
            className="rounded-2xl bg-[#d1d1d6] px-4 py-3 text-sm font-medium text-slate-950 transition hover:bg-white disabled:opacity-50"
          >
            {props.busy ? 'Cleaning...' : actionLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
