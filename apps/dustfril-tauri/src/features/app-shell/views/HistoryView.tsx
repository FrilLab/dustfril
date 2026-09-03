import { HistoryList } from '../../../components/HistoryList/HistoryList';
import type { ActivityRecord } from '../../../types/workflow';

type HistoryViewProps = {
  entries: ActivityRecord[];
};

export function HistoryView(props: HistoryViewProps) {
  return (
    <div className="history-view">
      <div className="content-heading">
        <div className="heading-icon heading-icon-history">◷</div>
        <div className="min-width-zero">
          <p className="eyebrow">History</p>
          <h1>Activity</h1>
        </div>
      </div>
      <HistoryList entries={props.entries} />
    </div>
  );
}
