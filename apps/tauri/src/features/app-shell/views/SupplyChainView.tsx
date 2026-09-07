import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import type { AsyncOperationState } from '../../../model/async';
import type {
  LifecycleScript,
  LockfileCheck,
  RiskLevel,
  SecurityFinding,
  SecurityScanResponse,
} from '../../../types/workflow';

type SupplyChainViewProps = {
  root: string;
  operation: AsyncOperationState<SecurityScanResponse>;
  canScan: boolean;
  onScan: () => void | Promise<void>;
};

export function SupplyChainView(props: SupplyChainViewProps) {
  const report = reportFromOperation(props.operation);
  const error = props.operation.status === 'error' ? props.operation.error : null;

  return (
    <div className="supply-chain-view">
      <header className="supply-chain-header">
        <div className="supply-chain-heading">
          <p className="eyebrow">Security</p>
          <h1>Supply Chain</h1>
          <p>
            Inspect local lifecycle scripts, dependency sources, and supported lockfiles with the
            offline Core analyzer. Scripts are never executed and no external service is contacted.
          </p>
          <p className="supply-chain-root" title={props.root}>
            Workspace: {props.root || 'No workspace selected'}
          </p>
        </div>
        <button
          type="button"
          className="supply-chain-scan-button"
          onClick={() => void props.onScan()}
          disabled={!props.canScan}
        >
          Scan Supply Chain
        </button>
      </header>

      {props.operation.status === 'idle' ? (
        <AsyncStatePanel
          status="idle"
          title="Ready for an explicit scan"
          description="Choose Scan Supply Chain to inspect the selected workspace. Navigating here does not start a scan."
        />
      ) : null}

      {props.operation.status === 'loading' ? (
        <AsyncStatePanel
          status="loading"
          title="Inspecting supply chain inputs"
          description="DustFril is reading supported manifests and lockfiles and applying offline rules."
        />
      ) : null}

      {props.operation.status === 'error' && !report ? (
        <AsyncStatePanel
          status="error"
          title="Supply-chain inspection failed"
          description="The required project input could not be inspected. Malformed or unreadable input is not treated as clean."
          error={error ?? 'The supply-chain scan failed.'}
        />
      ) : null}

      {report ? (
        <SupplyChainResults
          report={report}
          staleError={error}
          partial={props.operation.status === 'partial'}
        />
      ) : null}
    </div>
  );
}

function SupplyChainResults({
  report,
  staleError,
  partial,
}: {
  report: SecurityScanResponse;
  staleError: string | null;
  partial: boolean;
}) {
  const lifecycleScripts = report.lifecycleScripts ?? [];
  const findings = report.findings ?? [];
  const lifecycleFindings = findings.filter((finding) => finding.rule === 'suspicious-script');
  const dependencyFindings = findings.filter((finding) => finding.rule !== 'suspicious-script');
  const highestRisk = highestRiskLevel(findings);
  const hasSupportedInput =
    report.manifests.length > 0 || report.lockfiles.length > 0 || lifecycleScripts.length > 0;
  const hasUnsupportedScope =
    report.lockfiles.length === 0 &&
    report.manifests.some((path) => fileName(path) === 'package.json');
  const hasWarnings = partial || hasUnsupportedScope;
  const warnings = [
    report.historyWarning,
    hasUnsupportedScope
      ? 'No supported lockfile format was inspected for this package manager; the result is partial.'
      : undefined,
  ].filter((warning): warning is string => Boolean(warning));
  const context = scanContext(report, lifecycleScripts);

  return (
    <div className="supply-chain-results">
      {staleError ? (
        <div className="supply-chain-scan-error" role="status">
          The latest scan failed: {staleError}. Showing the previous result until another scan
          completes.
        </div>
      ) : null}

      {hasWarnings ? (
        <AsyncStatePanel
          status="partial"
          title="Scan completed with a warning"
          description="The security result is available, but part of the requested scope needs review."
          warnings={warnings}
        />
      ) : null}

      <section className="supply-chain-summary-grid" aria-label="Supply-chain scan summary">
        <SummaryCard
          value={
            !hasSupportedInput
              ? 'No input'
                : findings.length
                  ? 'Findings'
                : hasWarnings
                  ? 'Partial'
                  : 'No findings'
          }
          label="Scan result"
        />
        <SummaryCard value={findings.length} label="Findings" />
        <SummaryCard value={highestRisk ?? 'None'} label="Highest risk" />
        <SummaryCard value={context.join(' · ') || 'Unknown'} label="Ecosystem / manager" />
      </section>

      {!hasSupportedInput ? (
        <AsyncStatePanel
          status="empty"
          title="No supported project input found"
          description="No supported package manifest, lifecycle script, or lockfile was returned. This is not a clean security verdict."
        />
      ) : null}

      <section className="supply-chain-section" aria-labelledby="lifecycle-scripts-heading">
        <SectionHeading
          id="lifecycle-scripts-heading"
          title="Lifecycle scripts"
          description="Core-discovered install and lifecycle hooks. A listed script is not itself a malicious finding."
        />
        {lifecycleScripts.length ? (
          <div className="supply-chain-script-list">
            {lifecycleScripts.map((script, index) => (
              <LifecycleScriptCard
                key={`${script.package}-${script.scriptType}-${index}`}
                script={script}
                finding={findLifecycleFinding(script, lifecycleFindings)}
                source={sourceForScript(script, report)}
              />
            ))}
          </div>
        ) : (
          <p className="supply-chain-section-empty">No supported lifecycle hooks were found.</p>
        )}
      </section>

      <section className="supply-chain-section" aria-labelledby="lockfile-status-heading">
        <SectionHeading
          id="lockfile-status-heading"
          title="Lockfile status"
          description="Core-reported integrity state for supported lockfile formats."
        />
        {report.lockfiles.length ? (
          <div className="supply-chain-lockfile-table" role="table" aria-label="Lockfile status">
            <div className="supply-chain-lockfile-row supply-chain-lockfile-header" role="row">
              <span role="columnheader">Lockfile</span>
              <span role="columnheader">Format</span>
              <span role="columnheader">Status</span>
            </div>
            {report.lockfiles.map((lockfile) => (
              <LockfileRow key={`${lockfile.path}-${lockfile.kind}`} lockfile={lockfile} />
            ))}
          </div>
        ) : (
          <p className="supply-chain-section-empty">
            No supported lockfile was checked. Unsupported or not-applicable package managers are
            not reported as a missing npm lockfile.
          </p>
        )}
      </section>

      <section className="supply-chain-section" aria-labelledby="dependency-findings-heading">
        <SectionHeading
          id="dependency-findings-heading"
          title="Dependency source findings"
          description="Evidence-backed dependency and lockfile findings from the offline Core analysis."
        />
        {dependencyFindings.length ? (
          <div className="supply-chain-finding-list">
            {dependencyFindings.map((finding, index) => (
              <FindingCard finding={finding} key={`${finding.path}-${finding.rule}-${index}`} />
            ))}
          </div>
        ) : (
          <p className="supply-chain-section-empty">No dependency source or lockfile findings.</p>
        )}
      </section>

      {!findings.length && hasSupportedInput ? (
        <AsyncStatePanel
          status={hasWarnings ? 'partial' : 'success'}
          title={hasWarnings ? 'No findings in the analyzed inputs' : 'Scan completed with zero findings'}
          description={
            hasWarnings
              ? 'The analyzed inputs produced no findings, but the warning above still needs review.'
              : 'The inspected inputs produced no findings from the supported offline rules.'
          }
        />
      ) : null}
    </div>
  );
}

function LifecycleScriptCard({
  script,
  finding,
  source,
}: {
  script: LifecycleScript;
  finding: SecurityFinding | undefined;
  source: string;
}) {
  const risk = finding?.riskLevel ?? script.riskLevel;

  return (
    <article className={`supply-chain-card supply-chain-risk-${risk.toLowerCase()}`}>
      <div className="supply-chain-card-header">
        <div>
          <span className="supply-chain-card-kicker">{script.scriptType}</span>
          <h3>{script.package}</h3>
        </div>
        <span className="supply-chain-risk-badge">{finding ? risk : 'Safe'}</span>
      </div>
      <dl className="supply-chain-details">
        <div>
          <dt>Package manager</dt>
          <dd>{script.packageManager}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd title={source}>{source}</dd>
        </div>
        <div>
          <dt>Command</dt>
          <dd className="supply-chain-evidence">{script.command}</dd>
        </div>
      </dl>
      <p className="supply-chain-reason">
        {finding?.reason ?? 'No security finding was reported for this lifecycle hook.'}
      </p>
    </article>
  );
}

function LockfileRow({ lockfile }: { lockfile: LockfileCheck }) {
  return (
    <div className="supply-chain-lockfile-row" role="row">
      <span title={lockfile.path}>{fileName(lockfile.path)}</span>
      <span>{lockfileFormatLabel(lockfile.kind)}</span>
      <span className={`supply-chain-lockfile-status supply-chain-lockfile-${lockfile.status.toLowerCase()}`}>
        {lockfile.status}
      </span>
    </div>
  );
}

function FindingCard({ finding }: { finding: SecurityFinding }) {
  return (
    <article className={`supply-chain-card supply-chain-risk-${finding.riskLevel.toLowerCase()}`}>
      <div className="supply-chain-card-header">
        <div>
          <span className="supply-chain-card-kicker">{findingLabel(finding.rule)}</span>
          <h3>{finding.package ?? 'Workspace input'}</h3>
        </div>
        <span className="supply-chain-risk-badge">{finding.riskLevel}</span>
      </div>
      <dl className="supply-chain-details">
        <div>
          <dt>Source</dt>
          <dd title={finding.path}>{finding.path}</dd>
        </div>
        {finding.evidence ? (
          <div>
            <dt>Evidence</dt>
            <dd className="supply-chain-evidence">{finding.evidence}</dd>
          </div>
        ) : null}
      </dl>
      <p className="supply-chain-reason">{finding.reason}</p>
    </article>
  );
}

function SectionHeading({ id, title, description }: { id: string; title: string; description: string }) {
  return (
    <div className="supply-chain-section-heading">
      <h2 id={id}>{title}</h2>
      <p>{description}</p>
    </div>
  );
}

function SummaryCard({ value, label }: { value: number | string; label: string }) {
  return (
    <div className="supply-chain-summary-card">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function reportFromOperation(
  operation: AsyncOperationState<SecurityScanResponse>,
): SecurityScanResponse | undefined {
  if ('data' in operation) {
    return operation.data;
  }

  return operation.status === 'loading' || operation.status === 'error'
    ? operation.previous
    : undefined;
}

function findLifecycleFinding(script: LifecycleScript, findings: SecurityFinding[]) {
  return findings.find(
    (finding) =>
      finding.package === script.package &&
      finding.riskLevel === script.riskLevel &&
      finding.evidence === script.command,
  );
}

function sourceForScript(script: LifecycleScript, report: SecurityScanResponse) {
  const finding = report.findings.find(
    (candidate) =>
      candidate.rule === 'suspicious-script' &&
      candidate.package === script.package &&
      candidate.evidence === script.command,
  );
  return finding?.path ?? report.manifests.find((path) => fileName(path) === 'package.json') ?? 'package.json';
}

function scanContext(report: SecurityScanResponse, scripts: LifecycleScript[]) {
  const context = new Set<string>();
  if (report.manifests.some((path) => fileName(path) === 'package.json') || scripts.length) {
    context.add('Node');
  }
  if (report.manifests.some((path) => fileName(path) === 'Cargo.toml')) {
    context.add('Rust');
  }
  for (const script of scripts) {
    context.add(script.packageManager);
  }
  for (const lockfile of report.lockfiles) {
    if (lockfile.kind === 'PackageLockJson') context.add('npm');
    if (lockfile.kind === 'PnpmLockYaml') context.add('pnpm');
    if (lockfile.kind === 'BunLock') context.add('bun');
    if (lockfile.kind === 'CargoLock') context.add('Cargo');
  }
  return [...context];
}

function highestRiskLevel(findings: SecurityFinding[]): RiskLevel | null {
  const order: RiskLevel[] = ['None', 'Low', 'Medium', 'High', 'Critical'];
  return findings.reduce<RiskLevel | null>((highest, finding) => {
    if (!highest || order.indexOf(finding.riskLevel) > order.indexOf(highest)) {
      return finding.riskLevel;
    }
    return highest;
  }, null);
}

function findingLabel(rule: string) {
  return rule
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function lockfileFormatLabel(kind: LockfileCheck['kind']) {
  switch (kind) {
    case 'PackageLockJson':
      return 'npm';
    case 'PnpmLockYaml':
      return 'pnpm';
    case 'BunLock':
      return 'bun';
    case 'CargoLock':
      return 'Cargo';
  }
}

function fileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}
