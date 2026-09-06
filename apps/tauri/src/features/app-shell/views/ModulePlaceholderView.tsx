import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import type { CategoryConfig } from '../../../model/categories';

type ModulePlaceholderViewProps = {
  config: CategoryConfig;
  onReturnToOverview: () => void;
};

export function ModulePlaceholderView({ config, onReturnToOverview }: ModulePlaceholderViewProps) {
  return (
    <div className="module-placeholder-view">
      <div className="module-placeholder-heading">
        <p className="eyebrow">{config.section}</p>
        <h1>{config.title}</h1>
        <p>{config.description}</p>
      </div>
      <AsyncStatePanel
        status="unsupported"
        title={`${config.title} is planned`}
        description="This destination is reserved for an upcoming DustFril capability. No scan or analysis runs from this screen."
        actionLabel="Return to Overview"
        onAction={onReturnToOverview}
      />
    </div>
  );
}
