import { FolderIcon } from '../../../components/icons';
import { formatBytes, formatCount, formatDate } from '../../../lib/format';
import type { Ecosystem } from '../../../types/workflow';

type OverviewViewProps = {
  root: string;
  analysisReady: boolean;
  artifactCount: number;
  reclaimableBytes: number;
  lastAnalysisAtMs: number | null;
  historyCount: number;
  discoveredEcosystems: Ecosystem[];
  statusMessage: string;
  error: string | null;
};

export function OverviewView(props: OverviewViewProps) {
  return (
    <div className="overview-view">
      <div className="content-heading">
        <div className="heading-icon">
          <FolderIcon />
        </div>
        <div className="min-width-zero">
          <p className="eyebrow">Overview</p>
          <h1>Workspace summary</h1>
          <p className="heading-path" title={props.root}>
            {props.root || 'No workspace selected'}
          </p>
        </div>
      </div>

      <div className="overview-grid">
        <OverviewStat
          label="Artifacts"
          value={props.analysisReady ? formatCount(props.artifactCount) : 'Not analyzed'}
        />
        <OverviewStat
          label="Reclaimable"
          value={props.analysisReady ? formatBytes(props.reclaimableBytes) : 'Not analyzed'}
        />
        <OverviewStat
          label="Last analysis"
          value={props.lastAnalysisAtMs ? formatDate(props.lastAnalysisAtMs) : 'Not analyzed'}
        />
        <OverviewStat label="History entries" value={formatCount(props.historyCount)} />
      </div>

      <section className="overview-section">
        <div>
          <p className="eyebrow">Discovered ecosystems</p>
          <p className="overview-caption">
            DustFril identifies supported project artifacts during analysis.
          </p>
        </div>
        {props.discoveredEcosystems.length ? (
          <div className="ecosystem-list">
            {props.discoveredEcosystems.map((ecosystem) => (
              <span key={ecosystem} className="ecosystem-pill">
                {ecosystem}
              </span>
            ))}
          </div>
        ) : (
          <p className="overview-muted">No analysis results yet.</p>
        )}
      </section>

      <section className={`overview-status${props.error ? ' overview-status-error' : ''}`} role="status">
        <p className="eyebrow">Activity</p>
        <p>{props.statusMessage}</p>
      </section>
    </div>
  );
}

function OverviewStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="overview-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
