import { HistoryList } from '../../../components/HistoryList/HistoryList';
import type { CleanupHistoryEntry } from '../../../types/workflow';

type HistoryViewProps = {
  entries: CleanupHistoryEntry[];
};

export function HistoryView(props: HistoryViewProps) {
  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Cleanup History</p>
        <h2 className="mt-1 text-2xl font-semibold text-white">Previous Operations</h2>
        <p className="mt-2 text-sm text-slate-300">
          Review deleted paths, failures, freed storage, and cleanup mode for each completed run.
        </p>
      </section>

      <HistoryList entries={props.entries} />
    </div>
  );
}
