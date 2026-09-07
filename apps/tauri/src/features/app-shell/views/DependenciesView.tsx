import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import { formatCount } from '../../../lib/format';
import type { AsyncOperationStatus } from '../../../model/async';
import type {
  DependencyChange,
  DependencyDiff,
  DependencyEntry,
  DependencyInventoryResponse,
  DependencyMetric,
  DependencyReport,
} from '../../../types/workflow';

type DependenciesViewProps = {
  root: string;
  result: DependencyInventoryResponse | null;
  operationStatus: AsyncOperationStatus;
  busy: boolean;
  error: string | null;
  onLoad: () => void | Promise<void>;
  onCompare: () => void | Promise<void>;
  onAccept: () => void | Promise<void>;
};

export function DependenciesView(props: DependenciesViewProps) {
  const completeReportAvailable = Boolean(
    props.result?.inventoryFingerprint &&
      props.result.reports.some((report) => report.status === 'complete'),
  );

  return (
    <div className="dependencies-view">
      <div className="dependencies-heading">
        <div>
          <p className="eyebrow">Workspace</p>
          <h1>Dependencies</h1>
          <p className="dependencies-heading-path" title={props.root}>
            {props.root || 'No workspace selected'}
          </p>
        </div>
        <button
          type="button"
          className="button-secondary"
          onClick={() => void props.onLoad()}
          disabled={props.busy || !props.root}
        >
          Load Inventory
        </button>
      </div>

      <div className="dependencies-toolbar">
        <div>
          <strong>Read-only inventory</strong>
          <span>Manifests and lockfiles only. Installed dependency trees are not scanned.</span>
        </div>
        <div className="dependencies-actions">
          <button
            type="button"
            className="button-secondary"
            onClick={() => void props.onCompare()}
            disabled={props.busy || !props.root}
          >
            Compare Baseline
          </button>
          <button
            type="button"
            className="button-primary"
            onClick={() => void props.onAccept()}
            disabled={props.busy || !completeReportAvailable}
          >
            Accept Baseline
          </button>
        </div>
      </div>

      {props.error && props.operationStatus === 'error' ? (
        <div className="dependencies-notice dependencies-notice-error" role="alert">
          {props.error}
        </div>
      ) : null}

      {props.operationStatus === 'loading' && !props.result ? (
        <AsyncStatePanel
          status="loading"
          title="Loading dependency inventory"
          description="DustFril is reading supported manifests and lockfiles."
        />
      ) : props.result ? (
        <div className="dependencies-content">
          {props.operationStatus === 'loading' ? (
            <div className="dependencies-notice" role="status">
              Refreshing the inventory…
            </div>
          ) : null}
          <InventorySection reports={props.result.reports} />
          {props.result.diff ? <BaselineSection diff={props.result.diff} /> : null}
        </div>
      ) : props.operationStatus === 'error' ? (
        <AsyncStatePanel
          status="error"
          title="Dependency inventory failed"
          description="The inventory could not be read. A parse failure is not treated as zero dependencies."
          error={props.error ?? undefined}
        />
      ) : (
        <AsyncStatePanel
          status="idle"
          title="Inspect this workspace"
          description="Load the current dependency inventory, or compare it with the stored accepted baseline."
        />
      )}
    </div>
  );
}

function InventorySection({ reports }: { reports: DependencyReport[] }) {
  return (
    <section className="dependencies-section" aria-labelledby="dependency-inventory-heading">
      <div className="dependencies-section-heading">
        <div>
          <p className="eyebrow" id="dependency-inventory-heading">
            Current inventory
          </p>
          <p>Counts and package entries retain the distinctions reported by Core.</p>
        </div>
        <span className="dependencies-section-count">
          {formatCount(reports.length)} report{reports.length === 1 ? '' : 's'}
        </span>
      </div>
      <div className="dependency-report-grid">
        {reports.map((report) => (
          <DependencyReportCard key={report.ecosystem} report={report} />
        ))}
      </div>
    </section>
  );
}

function DependencyReportCard({ report }: { report: DependencyReport }) {
  return (
    <article className={`dependency-report dependency-report-${report.status}`}>
      <div className="dependency-report-header">
        <div>
          <p className="eyebrow">{report.ecosystem}</p>
          <h2>{reportStatusLabel(report.status)}</h2>
        </div>
        <span className="dependency-status-badge">{reportStatusLabel(report.status)}</span>
      </div>

      <dl className="dependency-context">
        <div>
          <dt>Manifest</dt>
          <dd title={report.manifest}>{report.manifest}</dd>
        </div>
        <div>
          <dt>Format</dt>
          <dd>{report.manifestFormat ?? 'Not reported'}</dd>
        </div>
        {report.lockfile ? (
          <div>
            <dt>Lockfile</dt>
            <dd title={report.lockfile.path}>
              {report.lockfile.kind ?? 'Unrecognized format'} · {lockfileStatusLabel(report.lockfile.status)}
            </dd>
          </div>
        ) : null}
      </dl>

      <div className="dependency-metrics">
        <MetricCard label="Unique direct" value={formatCount(report.directDependencyTotal)} />
        <MetricCard label="Resolved packages" metric={report.resolvedDependencyCount} />
        <MetricCard label="Transitive" metric={report.transitiveDependencyCount} />
      </div>

      <section className="dependency-category-section" aria-label={`${report.ecosystem} direct categories`}>
        <div className="dependency-subheading">
          <h3>Direct dependency categories</h3>
          <span>Declared in the manifest</span>
        </div>
        <div className="dependency-category-list">
          {Object.entries(report.directDependencyCounts).map(([category, count]) => (
            <div className="dependency-category-row" key={category}>
              <span>{category}</span>
              <strong>{formatCount(count)}</strong>
            </div>
          ))}
        </div>
      </section>

      <section className="dependency-duplicate-section" aria-label={`${report.ecosystem} duplicate versions`}>
        <div className="dependency-subheading">
          <h3>Duplicate resolved versions</h3>
          <span>{formatCount(report.duplicateVersions.length)} package(s)</span>
        </div>
        {report.duplicateVersions.length ? (
          <ul className="dependency-duplicate-list">
            {report.duplicateVersions.map((duplicate) => (
              <li key={duplicate.name}>
                <strong>{duplicate.name}</strong>
                <span>{duplicate.versions.join(' · ')}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="dependency-muted">No duplicate resolved versions reported.</p>
        )}
      </section>

      <section className="dependency-entry-section" aria-label={`${report.ecosystem} resolved packages`}>
        <div className="dependency-subheading">
          <h3>Resolved packages</h3>
          <span>{formatCount(report.resolvedDependencies.length)} entries</span>
        </div>
        {report.resolvedDependencies.length ? (
          <div className="dependency-entry-list" role="table" aria-label={`${report.ecosystem} resolved packages`}>
            <div className="dependency-entry-row dependency-entry-header" role="row">
              <span role="columnheader">Package</span>
              <span role="columnheader">Version</span>
              <span role="columnheader">Scope</span>
              <span role="columnheader">Source</span>
            </div>
            {report.resolvedDependencies.map((entry) => (
              <DependencyEntryRow entry={entry} key={entryKey(entry)} />
            ))}
          </div>
        ) : (
          <p className="dependency-muted">No resolved package entries are available.</p>
        )}
      </section>

      {report.warnings.length ? <WarningList warnings={report.warnings} /> : null}
    </article>
  );
}

function MetricCard({ label, metric, value }: { label: string; metric?: DependencyMetric; value?: string }) {
  const available = metric?.status === 'available' && metric.value !== null;

  return (
    <div className="dependency-metric-card">
      <span>{label}</span>
      <strong>{metric ? (available ? formatCount(metric.value ?? 0) : metricStatusLabel(metric.status)) : value}</strong>
      {metric && !available && metric.reason ? <small>{metric.reason}</small> : null}
    </div>
  );
}

function DependencyEntryRow({ entry }: { entry: DependencyEntry }) {
  return (
    <div className="dependency-entry-row" role="row">
      <strong title={entry.name}>{entry.name}</strong>
      <span>{entry.version}</span>
      <span className={`dependency-scope dependency-scope-${entry.scope}`}>{scopeLabel(entry.scope)}</span>
      <span title={entry.source ?? undefined}>{entry.source ?? 'Not reported'}</span>
    </div>
  );
}

function BaselineSection({ diff }: { diff: DependencyDiff }) {
  const groups: Array<{ label: string; changes: DependencyChange[]; className: string }> = [
    { label: 'Added', changes: diff.added, className: 'added' },
    { label: 'Removed', changes: diff.removed, className: 'removed' },
    { label: 'Version changed', changes: diff.versionChanges, className: 'version' },
    { label: 'Source changed', changes: diff.sourceChanges, className: 'source' },
  ];

  return (
    <section className="dependencies-section" aria-labelledby="dependency-baseline-heading">
      <div className="dependencies-section-heading">
        <div>
          <p className="eyebrow" id="dependency-baseline-heading">
            Baseline comparison
          </p>
          <p>{baselineSummary(diff)}</p>
        </div>
        <span className="dependencies-section-count">{baselineStatusLabel(diff.baselineStatus)}</span>
      </div>
      <div className="dependency-baseline-context">
        <span>Workspace identity</span>
        <code title={diff.workspaceId}>{diff.workspaceId}</code>
      </div>

      {diff.warnings.length ? <WarningList warnings={diff.warnings} /> : null}
      {diff.baselineStatus === 'unavailable' ? (
        <div className="dependencies-notice dependencies-notice-warning" role="status">
          Comparison is unavailable because no complete inventory was returned. The stored baseline was not changed.
        </div>
      ) : null}
      {!hasDependencyChanges(diff) ? (
        <p className="dependency-empty-diff">No logical dependency changes detected.</p>
      ) : (
        <div className="dependency-change-groups">
          {groups.map((group) =>
            group.changes.length ? (
              <section className={`dependency-change-group dependency-change-${group.className}`} key={group.label}>
                <div className="dependency-subheading">
                  <h3>{group.label}</h3>
                  <span>{formatCount(group.changes.length)}</span>
                </div>
                <div className="dependency-change-list">
                  {group.changes.map((change, index) => (
                    <DependencyChangeRow key={`${group.label}-${index}-${changeKey(change)}`} change={change} />
                  ))}
                </div>
              </section>
            ) : null,
          )}
        </div>
      )}
    </section>
  );
}

function DependencyChangeRow({ change }: { change: DependencyChange }) {
  const previous = change.previous;
  const current = change.current;
  const entry = current ?? previous;

  return (
    <div className="dependency-change-row">
      <div>
        <strong>{entry?.name ?? 'Unknown package'}</strong>
        <span>{entry ? `${entry.ecosystem} · ${scopeLabel(entry.scope)}` : 'No package context reported'}</span>
      </div>
      {previous && current ? (
        <div className="dependency-change-transition">
          <span>{dependencyLabel(previous)} → {dependencyLabel(current)}</span>
        </div>
      ) : (
        <div className="dependency-change-transition">
          <span>{entry ? dependencyLabel(entry) : 'No version reported'}</span>
        </div>
      )}
    </div>
  );
}

function WarningList({ warnings }: { warnings: string[] }) {
  return (
    <ul className="dependency-warning-list">
      {warnings.map((warning) => <li key={warning}>{warning}</li>)}
    </ul>
  );
}

function entryKey(entry: DependencyEntry) {
  return `${entry.ecosystem}-${entry.name}-${entry.version}-${entry.source ?? 'unknown'}-${entry.scope}`;
}

function changeKey(change: DependencyChange) {
  return `${change.current ? entryKey(change.current) : ''}-${change.previous ? entryKey(change.previous) : ''}`;
}

function dependencyLabel(entry: DependencyEntry) {
  return `${entry.version}${entry.source ? ` · ${entry.source}` : ''}`;
}

function reportStatusLabel(status: DependencyReport['status']) {
  switch (status) {
    case 'complete':
      return 'Complete';
    case 'missingLockfile':
      return 'Missing lockfile';
    case 'unsupported':
      return 'Unsupported';
  }
}

function metricStatusLabel(status: DependencyMetric['status']) {
  return status === 'unknown' ? 'Unknown' : 'Unsupported';
}

function lockfileStatusLabel(status: NonNullable<DependencyReport['lockfile']>['status']) {
  switch (status) {
    case 'parsed':
      return 'Parsed';
    case 'missing':
      return 'Missing';
    case 'unsupported':
      return 'Unsupported';
  }
}

function scopeLabel(scope: DependencyEntry['scope']) {
  switch (scope) {
    case 'direct':
      return 'Direct';
    case 'transitive':
      return 'Transitive';
    case 'unknown':
      return 'Unknown';
  }
}

function baselineStatusLabel(status: DependencyDiff['baselineStatus']) {
  switch (status) {
    case 'baselineCreated':
      return 'Baseline created';
    case 'compared':
      return 'Compared';
    case 'unavailable':
      return 'Unavailable';
  }
}

function baselineSummary(diff: DependencyDiff) {
  switch (diff.baselineStatus) {
    case 'baselineCreated':
      return 'The first complete observation established the baseline. No dependencies are treated as newly added.';
    case 'compared':
      return hasDependencyChanges(diff)
        ? 'Changes are shown below. Compare does not update the accepted baseline.'
        : 'The current logical inventory matches the accepted baseline.';
    case 'unavailable':
      return 'A complete inventory is required before the current state can be compared.';
  }
}

function hasDependencyChanges(diff: DependencyDiff) {
  return Boolean(
    diff.added.length ||
      diff.removed.length ||
      diff.versionChanges.length ||
      diff.sourceChanges.length,
  );
}
