import { useEffect } from 'react';
import { EmptyState } from '../../../components/EmptyState/EmptyState';
import { FolderIcon, ItemIcon } from '../../../components/icons';
import { formatAge, formatBytes, formatCount, formatDate } from '../../../lib/format';
import {
  artifactDetailLines,
  artifactLabel,
  leafName,
  recommendationClass,
  recommendationLabel,
} from '../../../model/presentation';
import type {
  ArtifactAnalysis,
  CleanupCandidate,
  DeleteMode,
} from '../../../types/workflow';
import { cleanupAgeOptions } from '../../../types/workflow';

type WorkspaceViewProps = {
  artifacts: ArtifactAnalysis[];
  artifactCount: number;
  candidates: CleanupCandidate[];
  reclaimableBytes: number;
  selectedItemId: string | null;
  selectedPaths: string[];
  deleteMode: DeleteMode;
  deleteModes: DeleteMode[];
  cleanupAgeDays: number;
  lastAnalysisAtMs: number | null;
  busy: boolean;
  analysisReady: boolean;
  error: string | null;
  onSelectItem: (path: string) => void;
  onCloseInspector: () => void;
  onTogglePath: (path: string) => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  onCleanupAgeChange: (days: number) => void | Promise<void>;
};

export function WorkspaceView(props: WorkspaceViewProps) {
  const candidatesByPath = new Map(props.candidates.map((candidate) => [candidate.path, candidate]));
  const selectedArtifact =
    props.artifacts.find((artifact) => artifact.path === props.selectedItemId) ?? null;

  useEffect(() => {
    if (!selectedArtifact) {
      return;
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        props.onCloseInspector();
      }
    }

    window.addEventListener('keydown', closeOnEscape);
    return () => window.removeEventListener('keydown', closeOnEscape);
  }, [props.onCloseInspector, props.selectedItemId]);

  return (
    <div className="workspace-view">
      <div className="workspace-summary-strip" aria-live="polite">
        {props.analysisReady ? (
          <>
            <span>
              <strong>{formatCount(props.artifactCount)}</strong> artifact
              {props.artifactCount === 1 ? '' : 's'}
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
            {props.lastAnalysisAtMs ? (
              <span className="summary-last-analysis">
                Analyzed {formatDate(props.lastAnalysisAtMs)}
              </span>
            ) : null}
          </>
        ) : (
          <span>Analyze this workspace to load artifacts and cleanup recommendations.</span>
        )}
      </div>

      {props.error ? (
        <div className="workspace-notice workspace-notice-warning" role="status">
          {props.error}
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
            <label className="cleanup-age-control">
              <span>Cleanup age</span>
              <select
                value={props.cleanupAgeDays}
                onChange={(event) => void props.onCleanupAgeChange(Number(event.currentTarget.value))}
                disabled={props.busy}
                aria-label="Cleanup age"
              >
                {cleanupAgeOptions.map((days) => (
                  <option key={days} value={days}>
                    {days} days
                  </option>
                ))}
              </select>
            </label>
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

        {selectedArtifact ? (
          <Inspector
            artifact={selectedArtifact}
            candidate={candidatesByPath.get(selectedArtifact.path)}
            selectedPaths={props.selectedPaths}
            onClose={props.onCloseInspector}
          />
        ) : null}
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
                aria-pressed={props.deleteMode === mode}
                onClick={() => props.onDeleteModeChange(mode)}
              >
                {mode === 'Trash' ? 'Move to Trash' : 'Delete permanently'}
              </button>
            ))}
          </div>
        </div>
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
        <span role="columnheader">Project</span>
        <span role="columnheader">Artifact</span>
        <span role="columnheader">Size</span>
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
                    onKeyDown={(event) => event.stopPropagation()}
                    aria-label={`Select ${artifactLabel(artifact)} in ${artifact.project.displayName} for cleanup`}
                  />
                ) : (
                  <span className="result-selection-empty" aria-hidden="true" />
                )}
              </span>
              <span className="result-project" role="cell">
                <strong>{artifact.project.displayName}</strong>
                <small title={artifact.project.root}>{artifact.project.root}</small>
              </span>
              <span className="result-name result-artifact" role="cell">
                <ItemIcon kind="folder" />
                <span className="result-name-copy">
                  <strong>{leafName(artifact.path)}</strong>
                  <small title={artifact.path}>
                    {artifactLabel(artifact)} · {artifact.path}
                  </small>
                </span>
              </span>
              <span role="cell" className="result-size">
                {formatBytes(artifact.sizeBytes)}
              </span>
              <span role="cell" className="result-modified modified-column">
                {artifact.lastModifiedMs === null ? formatAge(artifact.ageDays) : formatDate(artifact.lastModifiedMs)}
              </span>
              <span role="cell">
                <span className={recommendationClass(artifact.recommendation)}>
                  {recommendationLabel(artifact.recommendation)}
                </span>
                {selected && artifact.recommendation !== 'SafeToClean' ? (
                  <small className="recommendation-guidance">
                    {recommendationGuidance(artifact.recommendation)}
                  </small>
                ) : null}
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
  onClose,
}: {
  artifact: ArtifactAnalysis | null;
  candidate: CleanupCandidate | undefined;
  selectedPaths: string[];
  onClose: () => void;
}) {
  if (!artifact) {
    return null;
  }

  return (
    <aside className="inspector-pane workspace-drawer" aria-label="Artifact inspector">
      <div className="inspector-header">
        <span className="eyebrow">Inspector</span>
        <button type="button" className="inspector-close" onClick={onClose} aria-label="Close inspector">
          ×
        </button>
      </div>
      <div className="inspector-content">
        {candidate && selectedPaths.includes(candidate.path) &&
        artifact.recommendation !== 'SafeToClean' ? (
          <p className="recommendation-guidance" role="status">
            {recommendationGuidance(artifact.recommendation)}
          </p>
        ) : null}
        <div className="inspector-title-row">
          <ItemIcon kind="folder" large />
          <div className="min-width-zero">
            <strong className="inspector-title">{artifact.project.displayName}</strong>
            <span className="inspector-artifact">{artifactLabel(artifact)}</span>
            <span className={recommendationClass(artifact.recommendation)}>
              {recommendationLabel(artifact.recommendation)}
            </span>
          </div>
        </div>
        <dl className="inspector-details">
          {artifactDetailLines(
            artifact,
            candidate,
            candidate ? selectedPaths.includes(candidate.path) : false,
          ).map(([label, value]) => (
            <div key={label}>
              <dt>{label}</dt>
              <dd title={value}>{value}</dd>
            </div>
          ))}
        </dl>
      </div>
    </aside>
  );
}

function recommendationGuidance(recommendation: ArtifactAnalysis['recommendation']) {
  return recommendation === 'NeedsReview'
    ? 'This artifact is relatively recent. DustFril did not recommend cleaning it automatically.'
    : 'This artifact was recently modified. DustFril recommends keeping it.';
}
