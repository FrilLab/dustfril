import { useMemo, useState } from 'react';
import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
import type { AsyncOperationState } from '../../../model/async';
import type {
  IntegrityCheck,
  IntegrityScanResponse,
  IntegrityStatus,
  SignatureReport,
  SignatureStatus,
} from '../../../types/workflow';
import {
  defaultIntegrityTools,
  type ExecutableIntegrityState,
} from '../hooks/useExecutableIntegrity';

const integrityStatusLabels: Record<IntegrityStatus, string> = {
  newBaseline: 'New baseline',
  unchanged: 'Unchanged',
  contentChanged: 'Content changed',
  resolvedPathChanged: 'Resolved path changed',
  missing: 'Missing',
  inspectionFailed: 'Inspection failed',
};

const signatureStatusLabels: Record<SignatureStatus, string> = {
  valid: 'Valid',
  unsigned: 'Unsigned',
  invalid: 'Invalid',
  unsupported: 'Unsupported',
  inspectionFailed: 'Inspection failed',
};

const signaturePlatformLabels = {
  macos: 'macOS',
  windows: 'Windows',
  linux: 'Linux',
  other: 'Other',
} as const;

export function ExecutableIntegrityView({
  integrity,
}: {
  integrity: ExecutableIntegrityState;
}) {
  const [selectedTools, setSelectedTools] = useState(defaultIntegrityTools);
  const [customTool, setCustomTool] = useState('');
  const result = integrity.result;

  const toolChoices = useMemo(
    () => [
      ...defaultIntegrityTools,
      ...selectedTools.filter((tool) => !defaultIntegrityTools.includes(tool)),
    ],
    [selectedTools],
  );

  function toggleTool(tool: string) {
    setSelectedTools((current) =>
      current.includes(tool) ? current.filter((value) => value !== tool) : [...current, tool],
    );
  }

  function addCustomTool() {
    const tool = customTool.trim();
    if (!tool) {
      return;
    }

    setSelectedTools((current) => (current.includes(tool) ? current : [...current, tool]));
    setCustomTool('');
  }

  return (
    <div className="integrity-view">
      <header className="integrity-heading">
        <div>
          <p className="eyebrow">Security</p>
          <h1>Executable Integrity</h1>
          <p>
            Inspect the developer tools resolved from PATH and compare their bytes with the local
            baseline. Inspection reads metadata and bytes only; it never launches a tool.
          </p>
        </div>
      </header>

      <section className="integrity-controls" aria-label="Executable integrity scan controls">
        <div className="integrity-controls-heading">
          <div>
            <p className="control-label">TOOLS TO INSPECT</p>
            <p className="integrity-caption">
              Start with DustFril&apos;s supported developer-tool set, or choose a subset.
            </p>
          </div>
          <button
            type="button"
            className="button-secondary"
            onClick={() => setSelectedTools(defaultIntegrityTools)}
            disabled={integrity.busy}
          >
            Restore defaults
          </button>
        </div>

        <div className="integrity-tool-grid">
          {toolChoices.map((tool) => (
            <label className="integrity-tool-option" key={tool}>
              <input
                type="checkbox"
                checked={selectedTools.includes(tool)}
                onChange={() => toggleTool(tool)}
                disabled={integrity.busy}
              />
              <span>{tool}</span>
            </label>
          ))}
        </div>

        <div className="integrity-custom-tool">
          <label htmlFor="integrity-custom-tool">Additional tool name or path</label>
          <div>
            <input
              id="integrity-custom-tool"
              value={customTool}
              onChange={(event) => setCustomTool(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  addCustomTool();
                }
              }}
              placeholder="/path/to/tool or tool-name"
              disabled={integrity.busy}
              spellCheck={false}
            />
            <button
              type="button"
              className="button-secondary"
              onClick={addCustomTool}
              disabled={integrity.busy || !customTool.trim()}
            >
              Add
            </button>
          </div>
          <p>
            The value is passed as one structured identifier. It is never interpolated into a
            shell command or executed.
          </p>
        </div>

        <div className="integrity-scan-actions">
          <span>{selectedTools.length} tool{selectedTools.length === 1 ? '' : 's'} selected</span>
          <button
            type="button"
            className="button-confirm"
            onClick={() => void integrity.scan(selectedTools)}
            disabled={integrity.busy || selectedTools.length === 0}
          >
            {integrity.busy ? 'Scanning…' : 'Run integrity scan'}
          </button>
        </div>
      </section>

      <IntegrityScanContent operation={integrity.operation} result={result} />
    </div>
  );
}

function IntegrityScanContent({
  operation,
  result,
}: {
  operation: AsyncOperationState<IntegrityScanResponse>;
  result: IntegrityScanResponse | undefined;
}) {
  if (!result && operation.status === 'idle') {
    return (
      <AsyncStatePanel
        status="idle"
        title="Ready to inspect"
        description="Run an explicit scan to establish or compare the local executable-integrity baseline."
      />
    );
  }

  if (!result && operation.status === 'loading') {
    return (
      <AsyncStatePanel
        status="loading"
        title="Inspecting developer tools"
        description="DustFril is resolving paths, reading metadata, hashing bytes, and collecting supported signature evidence."
      />
    );
  }

  if (!result && operation.status === 'error') {
    return (
      <AsyncStatePanel
        status="error"
        title="Integrity scan failed"
        description="The executable-integrity scan could not be completed."
        error={operation.error}
      />
    );
  }

  if (!result || result.checks.length === 0) {
    return (
      <AsyncStatePanel
        status="empty"
        title="No tools inspected"
        description="Select at least one supported developer tool and run the scan again."
      />
    );
  }

  return (
    <section className="integrity-results" aria-label="Executable integrity results">
      {operation.status === 'loading' ? (
        <div className="integrity-inline-status" role="status">
          Comparing a new scan with the previous result…
        </div>
      ) : null}
      {operation.status === 'error' ? (
        <div className="integrity-inline-status integrity-inline-status-error" role="status">
          The latest scan failed: {operation.error}. The result below is from the previous completed scan.
        </div>
      ) : null}
      <IntegritySummary checks={result.checks} />
      <p className="integrity-interpretation">
        A changed path or hash is integrity evidence for review, not proof of malware or compromise.
        Unsigned, invalid, and unsupported signature states are reported as evidence only.
      </p>
      <p className="integrity-baseline-note">
        Core persists each successful observation as the next baseline. Missing or failed
        observations do not erase the last successful baseline.
      </p>
      <div className="integrity-check-list">
        {result.checks.map((check) => (
          <IntegrityCheckCard check={check} key={`${check.requestedTool}-${check.observation?.resolvedPath ?? check.status}`} />
        ))}
      </div>
    </section>
  );
}

function IntegritySummary({ checks }: { checks: IntegrityCheck[] }) {
  const changed = checks.filter((check) =>
    check.status === 'contentChanged' || check.status === 'resolvedPathChanged',
  ).length;
  const unavailable = checks.filter(
    (check) => check.status === 'missing' || check.status === 'inspectionFailed',
  ).length;
  const signatureReview = checks.filter((check) =>
    check.signature && ['unsigned', 'invalid', 'inspectionFailed'].includes(check.signature.status),
  ).length;

  return (
    <div className="integrity-summary-strip" aria-live="polite">
      <strong>{checks.length} tool{checks.length === 1 ? '' : 's'} checked</strong>
      <span>{changed} content/path change{changed === 1 ? '' : 's'}</span>
      <span>{unavailable} unavailable</span>
      <span>{signatureReview} signature state{signatureReview === 1 ? '' : 's'} to review</span>
    </div>
  );
}

function IntegrityCheckCard({ check }: { check: IntegrityCheck }) {
  const current = check.observation;
  const previous = check.previousObservation;

  return (
    <article className="integrity-check-card">
      <header className="integrity-check-heading">
        <div>
          <p className="control-label">REQUESTED TOOL</p>
          <h2>{check.requestedTool}</h2>
        </div>
        <span className={`integrity-status integrity-status-${check.status}`}>
          {integrityStatusLabels[check.status]}
        </span>
      </header>

      <div className="integrity-evidence-grid">
        <Evidence label="Resolved path" value={current?.resolvedPath ?? 'Not resolved'} />
        <Evidence label="Canonical target" value={current?.canonicalPath ?? 'Not available'} />
        {current?.symlinkTarget ? <Evidence label="Symlink target" value={current.symlinkTarget} /> : null}
        {current ? <HashEvidence label="Current SHA-256" value={current.sha256} /> : null}
        {previous ? <HashEvidence label="Previous SHA-256" value={previous.sha256} /> : null}
        {previous ? <Evidence label="Previous resolved path" value={previous.resolvedPath} /> : null}
        {current ? <Evidence label="Observed at" value={current.observedAt} /> : null}
        {current ? <Evidence label="Size" value={`${current.sizeBytes} bytes`} /> : null}
      </div>

      {check.failure ? (
        <div className="integrity-evidence-warning">
          <strong>{check.failure.kind}</strong>
          <span>{check.failure.message}</span>
        </div>
      ) : null}

      <SignatureEvidence report={check.signature} />
    </article>
  );
}

function SignatureEvidence({ report }: { report?: SignatureReport }) {
  if (!report) {
    return (
      <div className="integrity-signature integrity-signature-none">
        <div>
          <p className="control-label">SIGNATURE EVIDENCE</p>
          <p>No signature result is available because the target could not be inspected.</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`integrity-signature integrity-signature-${report.status}`}>
      <div className="integrity-signature-heading">
        <div>
          <p className="control-label">
            SIGNATURE EVIDENCE · {signaturePlatformLabels[report.platform]}
          </p>
          <strong>{signatureStatusLabels[report.status]}</strong>
        </div>
        <span className={`integrity-status integrity-status-signature-${report.status}`}>
          {signatureStatusLabels[report.status]}
        </span>
      </div>
      <div className="integrity-evidence-grid">
        {report.signer ? <Evidence label="Signer / publisher" value={report.signer} /> : null}
        {report.teamIdentifier ? <Evidence label="Team identifier" value={report.teamIdentifier} /> : null}
        {report.verificationCode !== undefined ? (
          <Evidence label="Verification code" value={String(report.verificationCode)} />
        ) : null}
        {report.verificationMessage ? <Evidence label="Verifier message" value={report.verificationMessage} /> : null}
      </div>
      {report.failure ? (
        <p className="integrity-signature-detail">
          {report.failure.kind}: {report.failure.message}
        </p>
      ) : null}
    </div>
  );
}

function Evidence({ label, value }: { label: string; value: string }) {
  return (
    <div className="integrity-evidence">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function HashEvidence({ label, value }: { label: string; value: string }) {
  return (
    <div className="integrity-evidence">
      <dt>{label}</dt>
      <dd>
        <code className="integrity-hash" title={value}>
          {value}
        </code>
      </dd>
    </div>
  );
}
