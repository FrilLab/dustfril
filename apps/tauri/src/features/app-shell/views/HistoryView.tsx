import { useState } from 'react';
import { ClearHistoryDialog } from '../../../components/ClearHistoryDialog/ClearHistoryDialog';
import { HistoryList } from '../../../components/HistoryList/HistoryList';
import { formatCount } from '../../../lib/format';
import type { ActivityRecord } from '../../../types/workflow';

type HistoryViewProps = {
  entries: ActivityRecord[];
  busy: boolean;
  error: string | null;
  onClearHistory: () => void | Promise<void>;
};

export function HistoryView(props: HistoryViewProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);

  async function handleConfirmClear() {
    try {
      await props.onClearHistory();
      setConfirmOpen(false);
    } catch {
      // The app state owns and displays the command error. Keep the dialog
      // open so a failed clear cannot look like a successful one.
    }
  }

  return (
    <div className="history-view">
      <div className="history-toolbar">
        <span className="history-toolbar-count" aria-live="polite">
          {formatCount(props.entries.length)} activit{props.entries.length === 1 ? 'y' : 'ies'}
        </span>
        <button
          type="button"
          className="clear-history-button"
          onClick={() => setConfirmOpen(true)}
          disabled={props.busy || props.entries.length === 0}
        >
          Clear History
        </button>
      </div>
      {props.error ? (
        <div className="history-notice" role="status">
          {props.error}
        </div>
      ) : null}
      <HistoryList entries={props.entries} />
      <ClearHistoryDialog
        open={confirmOpen}
        busy={props.busy}
        error={props.error}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={handleConfirmClear}
      />
    </div>
  );
}
