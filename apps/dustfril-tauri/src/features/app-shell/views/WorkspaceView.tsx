import { EmptyState } from '../../../components/EmptyState/EmptyState';
import { FolderIcon, ItemIcon } from '../../../components/icons';
import { formatAge, formatBytes, formatCount, formatDate } from '../../../lib/format';
import {
  artifactDetailLines,
  kindForArtifact,
  leafName,
  recommendationClass,
  recommendationLabel,
} from '../../../model/presentation';
import type {
  ArtifactAnalysis,
  CleanupCandidate,
  DeleteMode,
  Ecosystem,
} from '../../../types/workflow';

type WorkspaceViewProps = {
  root: string;
  artifacts: ArtifactAnalysis[];
  candidates: CleanupCandidate[];
  reclaimableBytes: number;
  selectedItemId: string | null;
  selectedPaths: string[];
  selectedBytes: number;
  deleteMode: DeleteMode;
  deleteModes: DeleteMode[];
  lastAnalysisAtMs: number | null;
  busy: boolean;
  analysisReady: boolean;
  statusMessage: string;
  error: string | null;
  discoveredEcosystems: Ecosystem[];
  onSelectItem: (path: string) => void;
  onTogglePath: (path: string) => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
};

export function WorkspaceView(props: WorkspaceViewProps) {
  const candidatesByPath = new Map(props.candidates.map((candidate) => [candidate.path, candidate]));
  const selectedArtifact =
    props.artifacts.find((artifact) => artifact.path === props.selectedItemId) ?? null;

  return (
    <div className="workspace-view">
      <div className="content-heading">
        <div className="heading-icon">
          <FolderIcon />
        </div>
        <div className="min-width-zero">
          <p className="eyebrow">Workspace</p>
          <h1>{props.root ? leafName(props.root) : 'Choose a workspace'}</h1>
          <p className="heading-path" title={props.root}>
            {props.root || 'Select a folder to discover supported project artifacts.'}
          </p>
        </div>
        {props.lastAnalysisAtMs ? (
          <p className="last-analysis">Analyzed {formatDate(props.lastAnalysisAtMs)}</p>
        ) : null}
      </div>

      <div className="workspace-summary-strip" aria-live="polite">
        {props.analysisReady ? (
          <>
            <span>
              <strong>{formatCount(props.artifacts.length)}</strong> artifact
              {props.artifacts.length === 1 ? '' : 's'}
            </span>
            <span className="summary-separator">·</span>
            <span>
              <strong>
                {formatBytes(props.reclaimableBytes)}
              </strong>{' '}
              reclaimable
            </span>
            <span className="summary-separator">·</span>
            <span>
              <strong>{formatCount(props.selectedPaths.length)}</strong> selected
            </span>
          </>
        ) : (
          <span>Analyze this workspace to load artifacts and cleanup recommendations.</span>
        )}
        <div className="ecosystem-list">
          {props.discoveredEcosystems.map((ecosystem) => (
            <span key={ecosystem} className="ecosystem-pill">
              {ecosystem}
            </span>
          ))}
        </div>
      </div>

      {props.error ? (
        <div className="workspace-notice workspace-notice-warning" role="status">
          {props.error}
        </div>
      ) : null}

      {!props.error && props.analysisReady ? (
        <div className="workspace-notice" role="status">
          {props.statusMessage}
        </div>
      ) : null}

      <div className="workspace-layout">
        <section className="results-pane" aria-label="Workspace artifacts">
          <div className="results-toolbar">
            <div>
              <p className="eyebrow">Cleanup recommendations</p>
              <p className="results-caption">
                {props.analysisReady
                  ? 'Select candidates to include in the cleanup review.'
                  : 'Analyze the workspace to load recommendations.'}
              </p>
            </div>
            {props.analysisReady ? (
              <span className="results-count">{formatCount(props.artifacts.length)} visible</span>
            ) : null}
          </div>

          <WorkspaceResults
            artifacts={props.artifacts}
            candidatesByPath={candidatesByPath}
            selectedItemId={props.selectedItemId}
            selectedPaths={props.selectedPaths}
            analysisReady={props.analysisReady}
            onSelectItem={props.onSelectItem}
            onTogglePath={props.onTogglePath}
          />
        </section>

        <Inspector artifact={selectedArtifact} candidate={selectedArtifact ? candidatesByPath.get(selectedArtifact.path) : undefined} selectedPaths={props.selectedPaths} />
      </div>

      <div className="workspace-controls">
        <div className="cleanup-mode-control">
          <span className="control-label">Cleanup mode</span>
          <div className="mode-toggle" role="group" aria-label="Cleanup mode">
            {props.deleteModes.map((mode) => (
              <button
                key={mode}
                type="button"
                className={props.deleteMode === mode ? 'mode-active' : ''}
                onClick={() => props.onDeleteModeChange(mode)}
              >
                {mode === 'Trash' ? 'Move to Trash' : 'Delete permanently'}
              </button>
            ))}
          </div>
        </div>
        <p className="workspace-control-note">
          {props.busy
            ? 'Analysis in progress…'
            : `${formatCount(props.selectedPaths.length)} selected · ${formatBytes(props.selectedBytes)}`}
        </p>
      </div>
    </div>
  );
}

type WorkspaceResultsProps = {
  artifacts: ArtifactAnalysis[];
  candidatesByPath: Map<string, CleanupCandidate>;
  selectedItemId: string | null;
  selectedPaths: string[];
  analysisReady: boolean;
  onSelectItem: (path: string) => void;
  onTogglePath: (path: string) => void;
};

function WorkspaceResults(props: WorkspaceResultsProps) {
  if (!props.analysisReady) {
    return (
      <EmptyState
        icon={<FolderIcon />}
        message="Choose a workspace folder and click Analyze Workspace to find Rust, Node, and Java artifacts."
      />
    );
  }

  if (!props.artifacts.length) {
    return (
      <EmptyState
        icon={<FolderIcon />}
        message="No supported development artifacts were found in this workspace."
      />
    );
  }

  return (
    <div className="workspace-table" role="table" aria-label="Cleanup recommendations">
      <div className="result-row result-row-header" role="row">
        <span aria-hidden="true" />
        <span role="columnheader">Name</span>
        <span role="columnheader">Size</span>
        <span role="columnheader">Kind</span>
        <span role="columnheader" className="modified-column">
          Modified
        </span>
        <span role="columnheader">Status</span>
      </div>
      <div className="workspace-table-body">
        {props.artifacts.map((artifact) => {
          const candidate = props.candidatesByPath.get(artifact.path);
          const selected = candidate ? props.selectedPaths.includes(candidate.path) : false;
          const active = artifact.path === props.selectedItemId;

          return (
            <div
              key={artifact.path}
              className={`result-row${active ? ' result-row-active' : ''}`}
              role="row"
              tabIndex={0}
              onClick={() => props.onSelectItem(artifact.path)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                  event.preventDefault();
                  props.onSelectItem(artifact.path);
                }
              }}
            >
              <span className="result-selection" role="cell">
                {candidate ? (
                  <input
                    type="checkbox"
                    checked={selected}
                    onChange={() => props.onTogglePath(candidate.path)}
                    onClick={(event) => event.stopPropagation()}
                    aria-label={`Select ${leafName(artifact.path)} for cleanup`}
                  />
                ) : (
                  <span className="result-selection-empty" aria-hidden="true" />
                )}
              </span>
              <span className="result-name" role="cell">
                <ItemIcon kind="folder" />
                <span className="result-name-copy">
                  <strong>{leafName(artifact.path)}</strong>
                  <small title={artifact.path}>{artifact.path}</small>
                </span>
              </span>
              <span role="cell" className="result-size">
                {formatBytes(artifact.sizeBytes)}
              </span>
              <span role="cell" className="result-kind">
                {kindForArtifact(artifact.ecosystem)}
              </span>
              <span role="cell" className="result-modified modified-column">
                {artifact.lastModifiedMs === null ? formatAge(artifact.ageDays) : formatDate(artifact.lastModifiedMs)}
              </span>
              <span role="cell">
                <span className={recommendationClass(artifact.recommendation)}>
                  {recommendationLabel(artifact.recommendation)}
                </span>
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function Inspector({
  artifact,
  candidate,
  selectedPaths,
}: {
  artifact: ArtifactAnalysis | null;
  candidate: CleanupCandidate | undefined;
  selectedPaths: string[];
}) {
  return (
    <aside className="inspector-pane" aria-label="Artifact inspector">
      <div className="inspector-header">
        <span className="eyebrow">Inspector</span>
      </div>
      {artifact ? (
        <div className="inspector-content">
          <div className="inspector-title-row">
            <ItemIcon kind="folder" large />
            <div className="min-width-zero">
              <strong className="inspector-title">{leafName(artifact.path)}</strong>
              <span className={recommendationClass(artifact.recommendation)}>
                {recommendationLabel(artifact.recommendation)}
              </span>
            </div>
          </div>
          <dl className="inspector-details">
            {artifactDetailLines(artifact, candidate, candidate ? selectedPaths.includes(candidate.path) : false).map(
              ([label, value]) => (
                <div key={label}>
                  <dt>{label}</dt>
                  <dd title={value}>{value}</dd>
                </div>
              ),
            )}
          </dl>
        </div>
      ) : (
        <EmptyState compact message="Select an artifact to inspect its path, size, and recommendation." />
      )}
    </aside>
  );
}
