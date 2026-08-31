import { HistoryList } from '../../../components/HistoryList/HistoryList';
import type { ActivityRecord } from '../../../types/workflow';

type HistoryViewProps = {
  entries: ActivityRecord[];
};

export function HistoryView(props: HistoryViewProps) {
  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Activity History</p>
        <h2 className="mt-1 text-2xl font-semibold text-white">Operation Log</h2>
        <p className="mt-2 text-sm text-slate-300">
          Review scans, cleanup operations, results, and failures recorded by DustFril.
        </p>
      </section>

      <HistoryList entries={props.entries} />
    </div>
  );
}
