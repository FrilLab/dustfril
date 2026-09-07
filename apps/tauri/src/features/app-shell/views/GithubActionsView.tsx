import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import { ItemIcon } from '../../../components/icons';
import type { AsyncOperationState } from '../../../model/async';
import type {
  RiskLevel,
  WorkflowFinding,
  WorkflowFindingCategory,
  WorkflowScanResponse,
} from '../../../types/workflow';

type GithubActionsViewProps = {
  root: string;
  operation: AsyncOperationState<WorkflowScanResponse>;
  canScan: boolean;
  onScan: () => void | Promise<void>;
};

export function GithubActionsView(props: GithubActionsViewProps) {
  const report = reportFromOperation(props.operation);
  const error = props.operation.status === 'error' ? props.operation.error : null;

  return (
    <div className="github-actions-view">
      <header className="github-actions-header">
        <div className="github-actions-heading">
          <p className="eyebrow">Security</p>
          <h1>GitHub Actions</h1>
          <p>
            Inspect local workflow files with the offline Core analyzer. Nothing is executed,
            uploaded, or resolved from GitHub.
          </p>
          <p className="github-actions-root" title={props.root}>
            Workspace: {props.root || 'No workspace selected'}
          </p>
        </div>
        <button
          type="button"
          className="workflow-scan-button"
          onClick={() => void props.onScan()}
          disabled={!props.canScan}
        >
          Scan Workflows
        </button>
      </header>

      {props.operation.status === 'idle' ? (
        <AsyncStatePanel
          status="idle"
          title="Ready for an explicit scan"
          description="Choose Scan Workflows to inspect only .github/workflows/*.yml and *.yaml files in the selected workspace."
        />
      ) : null}

      {props.operation.status === 'loading' ? (
        <AsyncStatePanel
          status="loading"
          title="Inspecting workflows"
          description="DustFril is parsing local workflow structure and applying supported static rules."
        />
      ) : null}

      {props.operation.status === 'error' && !report ? (
        <AsyncStatePanel
          status="error"
          title="Workflow inspection failed"
          description="The workflow files could not be inspected. Malformed or unreadable input is not treated as clean."
          error={error ?? 'The workflow scan failed.'}
        />
      ) : null}

      {report ? (
        <WorkflowResults report={report} staleError={error} operationStatus={props.operation.status} />
      ) : null}
    </div>
  );
}

function WorkflowResults({
  report,
  staleError,
  operationStatus,
}: {
  report: WorkflowScanResponse;
  staleError: string | null;
  operationStatus: AsyncOperationState<WorkflowScanResponse>['status'];
}) {
  if (!report.workflows.length) {
    return (
      <div className="github-actions-results">
        {staleError ? <ScanErrorNotice error={staleError} /> : null}
        <AsyncStatePanel
          status={report.notices.length ? 'partial' : 'empty'}
          title="No workflow files found"
          description="This workspace has no direct .github/workflows/*.yml or *.yaml files. That is an empty inspection result, not a clean security verdict."
          warnings={report.notices.map(formatNotice)}
        />
      </div>
    );
  }

  const findingsByCategory = groupFindings(report.findings);
  const jobCount = report.workflows.reduce((total, workflow) => total + workflow.jobs.length, 0);
  const highestRisk = highestRiskLevel(report.findings);
  const partial = report.notices.length > 0 || operationStatus === 'partial';

  return (
    <div className="github-actions-results">
      {staleError ? <ScanErrorNotice error={staleError} /> : null}
      {partial ? (
        <AsyncStatePanel
          status="partial"
          title="Analysis is partial"
          description="Some workflow semantics are outside the supported offline rules. The notices below do not claim those paths are safe or leaked."
          warnings={report.notices.map(formatNotice)}
        />
      ) : null}

      <section className="workflow-summary-grid" aria-label="Workflow scan summary">
        <SummaryCard value={report.workflows.length} label="Workflows inspected" />
        <SummaryCard value={jobCount} label="Jobs parsed" />
        <SummaryCard value={report.findings.length} label="Supported findings" />
        <SummaryCard value={highestRisk ?? 'None'} label="Highest risk" />
      </section>

      <section className="workflow-section" aria-labelledby="workflow-overview-heading">
        <SectionHeading
          id="workflow-overview-heading"
          title="Workflow overview"
          description="Every listed workflow was parsed successfully by Core."
        />
        <div className="workflow-overview-list">
          {report.workflows.map((workflow) => {
            const findingCount = report.findings.filter(
              (finding) => finding.workflowPath === workflow.path,
            ).length;
            const noticeCount = report.notices.filter(
              (notice) => notice.workflowPath === workflow.path,
            ).length;

            return (
              <article className="workflow-overview-card" key={workflow.path}>
                <div className="workflow-overview-icon">
                  <ItemIcon kind={findingCount ? 'warning' : 'document'} />
                </div>
                <div className="workflow-overview-copy">
                  <strong>{workflow.name || workflow.path.split(/[\\/]/).pop()}</strong>
                  <span title={workflow.path}>{workflow.path}</span>
                  <small>
                    {workflow.jobs.length} job{workflow.jobs.length === 1 ? '' : 's'} ·{' '}
                    {workflow.jobs.reduce((total, job) => total + job.stepCount, 0)} steps ·{' '}
                    {findingCount} finding{findingCount === 1 ? '' : 's'}
                    {noticeCount ? ` · ${noticeCount} notice${noticeCount === 1 ? '' : 's'}` : ''}
                  </small>
                </div>
                <span className="workflow-analysis-status">Analyzed</span>
              </article>
            );
          })}
        </div>
      </section>

      <FindingSection
        title="Command findings"
        description="Supported suspicious run commands reported by Core."
        category="suspiciousCommand"
        findings={findingsByCategory.suspiciousCommand}
      />
      <FindingSection
        title="Permission findings"
        description="Effective workflow/job token permissions, including exact write scopes."
        category="tokenPermissions"
        findings={findingsByCategory.tokenPermissions}
      />
      <FindingSection
        title="Secret-exposure findings"
        description="Only direct references reaching supported stdout or network sinks are reported as findings."
        category="secretExposure"
        findings={findingsByCategory.secretExposure}
      />

      {!report.findings.length ? (
        <AsyncStatePanel
          status={partial ? 'partial' : 'success'}
          title={partial ? 'No supported findings in the analyzed paths' : 'No supported findings'}
          description={
            partial
              ? 'The analyzed portions produced no findings; review the partial-analysis notices before treating the result as complete.'
              : 'The inspected workflows produced no findings from the supported local rules.'
          }
        />
      ) : null}
    </div>
  );
}

function FindingSection({
  title,
  description,
  category,
  findings,
}: {
  title: string;
  description: string;
  category: WorkflowFindingCategory;
  findings: WorkflowFinding[];
}) {
  return (
    <section className="workflow-section" aria-labelledby={`${category}-heading`}>
      <SectionHeading id={`${category}-heading`} title={title} description={description} />
      {findings.length ? (
        <div className="workflow-finding-list">
          {findings.map((finding, index) => (
            <FindingCard finding={finding} key={`${finding.workflowPath}-${finding.ruleId}-${index}`} />
          ))}
        </div>
      ) : (
        <p className="workflow-section-empty">No findings in this category.</p>
      )}
    </section>
  );
}

function FindingCard({ finding }: { finding: WorkflowFinding }) {
  const secretFinding = finding.category === 'secretExposure';
  const step = finding.stepIndex === undefined
    ? null
    : `${finding.stepName || 'Step'} (${finding.stepIndex + 1})`;

  return (
    <article className={`workflow-finding-card workflow-risk-${finding.riskLevel.toLowerCase()}`}>
      <div className="workflow-finding-header">
        <div>
          <span className="workflow-finding-category">{categoryLabel(finding.category)}</span>
          <h3>{finding.ruleId}</h3>
        </div>
        <span className="workflow-risk-badge">{finding.riskLevel}</span>
      </div>
      <dl className="workflow-finding-details">
        <div>
          <dt>Workflow</dt>
          <dd title={finding.workflowPath}>{finding.workflowPath}</dd>
        </div>
        <div>
          <dt>Job</dt>
          <dd>{finding.jobId || 'Workflow scope'}</dd>
        </div>
        {step ? (
          <div>
            <dt>Step</dt>
            <dd>{step}</dd>
          </div>
        ) : null}
        {secretFinding ? (
          <>
            <div>
              <dt>Secret reference</dt>
              <dd>{finding.secretReference || 'Reference name unavailable'}</dd>
            </div>
            <div>
              <dt>Known sink</dt>
              <dd>{sinkLabel(finding.exposureSink)}</dd>
            </div>
          </>
        ) : null}
        {finding.evidence ? (
          <div>
            <dt>{finding.category === 'tokenPermissions' ? 'Permission evidence' : 'Command context'}</dt>
            <dd className="workflow-evidence">{finding.evidence}</dd>
          </div>
        ) : null}
      </dl>
      <p className="workflow-finding-reason">{finding.reason}</p>
    </article>
  );
}

function SectionHeading({ id, title, description }: { id: string; title: string; description: string }) {
  return (
    <div className="workflow-section-heading">
      <div>
        <h2 id={id}>{title}</h2>
        <p>{description}</p>
      </div>
    </div>
  );
}

function SummaryCard({ value, label }: { value: number | string; label: string }) {
  return (
    <div className="workflow-summary-card">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function ScanErrorNotice({ error }: { error: string }) {
  return (
    <div className="workflow-scan-error" role="status">
      The latest scan failed: {error}. Showing the previous result until another scan completes.
    </div>
  );
}

function reportFromOperation(
  operation: AsyncOperationState<WorkflowScanResponse>,
): WorkflowScanResponse | undefined {
  if ('data' in operation) {
    return operation.data;
  }

  return operation.status === 'loading' || operation.status === 'error'
    ? operation.previous
    : undefined;
}

function groupFindings(findings: WorkflowFinding[]) {
  return findings.reduce(
    (groups, finding) => {
      groups[finding.category].push(finding);
      return groups;
    },
    {
      suspiciousCommand: [],
      tokenPermissions: [],
      secretExposure: [],
    } as Record<WorkflowFindingCategory, WorkflowFinding[]>,
  );
}

function highestRiskLevel(findings: WorkflowFinding[]): RiskLevel | null {
  const order: RiskLevel[] = ['None', 'Low', 'Medium', 'High', 'Critical'];
  return findings.reduce<RiskLevel | null>((highest, finding) => {
    if (!highest || order.indexOf(finding.riskLevel) > order.indexOf(highest)) {
      return finding.riskLevel;
    }
    return highest;
  }, null);
}

function categoryLabel(category: WorkflowFindingCategory) {
  switch (category) {
    case 'suspiciousCommand':
      return 'Suspicious command';
    case 'tokenPermissions':
      return 'Token permissions';
    case 'secretExposure':
      return 'Secret exposure';
  }
}

function sinkLabel(sink: WorkflowFinding['exposureSink']) {
  switch (sink) {
    case 'stdout':
      return 'stdout / logging';
    case 'networkRequest':
      return 'network request';
    default:
      return 'Supported sink';
  }
}

function formatNotice(notice: WorkflowScanResponse['notices'][number]) {
  return `${notice.workflowPath}${notice.jobId ? ` · job ${notice.jobId}` : ''}: ${notice.reason}`;
}
