import { HistoryList } from '../../../components/HistoryList/HistoryList';
import type { ActivityRecord } from '../../../types/workflow';

type HistoryViewProps = {
  entries: ActivityRecord[];
};

export function HistoryView(props: HistoryViewProps) {
  return (
    <div className="history-view">
      <HistoryList entries={props.entries} />
    </div>
  );
}
