import { useEffect, useMemo, useState } from 'react';
import { EmptyState } from '../../../components/EmptyState/EmptyState';
import { FolderIcon, ItemIcon } from '../../../components/icons';
import { SortableHeader } from '../../../components/SortableHeader/SortableHeader';
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
import {
  sortArtifacts,
  type WorkspaceSortColumn,
  type WorkspaceSortState,
} from '../../../model/sorting';

type WorkspaceViewProps = {
  artifacts: ArtifactAnalysis[];
  artifactCount: number;
  totalSizeBytes: number;
  candidates: CleanupCandidate[];
  selectedItemId: string | null;
  selectedPaths: string[];
  selectedCandidateBytes: number;
  canReviewCleanup: boolean;
  deleteMode: DeleteMode;
  deleteModes: DeleteMode[];
  cleanupAgeDays: number;
  busy: boolean;
  analysisReady: boolean;
  error: string | null;
  onSelectItem: (path: string) => void;
  onCloseInspector: () => void;
  onTogglePath: (path: string) => void;
  onDeleteModeChange: (mode: DeleteMode) => void;
  onCleanupAgeChange: (days: number) => void | Promise<void>;
  onRequestCleanup: () => void;
};

export function WorkspaceView(props: WorkspaceViewProps) {
  const [sort, setSort] = useState<WorkspaceSortState>({ column: 'size', direction: 'desc' });
  const candidatesByPath = new Map(props.candidates.map((candidate) => [candidate.path, candidate]));
  const selectedArtifact =
    props.artifacts.find((artifact) => artifact.path === props.selectedItemId) ?? null;
  const sortedArtifacts = useMemo(
    () => sortArtifacts(props.artifacts, sort),
    [props.artifacts, sort],
  );

  function handleSort(column: WorkspaceSortColumn) {
    setSort((current) => ({
      column,
      direction: current.column === column && current.direction === 'asc' ? 'desc' : 'asc',
    }));
  }

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
        <span>
          <strong>
            {formatCount(props.artifactCount)} artifact{props.artifactCount === 1 ? '' : 's'}
          </strong>{' '}
          ·{' '}
          <strong>{formatBytes(props.totalSizeBytes)}</strong> total
        </span>
        <label className="cleanup-age-control">
          <span>AGE</span>
          <select
            value={props.cleanupAgeDays}
            onChange={(event) => void props.onCleanupAgeChange(Number(event.currentTarget.value))}
            disabled={props.busy}
            aria-label="AGE"
          >
            {cleanupAgeOptions.map((days) => (
              <option key={days} value={days}>
                {days} days
              </option>
            ))}
          </select>
        </label>
      </div>

      {props.error ? (
        <div className="workspace-notice workspace-notice-warning" role="status">
          {props.error}
        </div>
      ) : null}

      <div className="workspace-layout">
        <section className="results-pane" aria-label="Workspace artifacts">
          <WorkspaceResults
            artifacts={sortedArtifacts}
            candidatesByPath={candidatesByPath}
            selectedItemId={props.selectedItemId}
            selectedPaths={props.selectedPaths}
            analysisReady={props.analysisReady}
            sort={sort}
            onSort={handleSort}
            onSelectItem={props.onSelectItem}
            onTogglePath={props.onTogglePath}
          />

          <div className="workspace-controls">
            <div className="cleanup-mode-control">
              <span className="control-label">MODE</span>
              <div className="mode-toggle" role="group" aria-label="MODE">
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
            <div className="cleanup-selection-summary">
              <span>
                {formatCount(props.selectedPaths.length)} selected · {formatBytes(props.selectedCandidateBytes)}
              </span>
              <button
                type="button"
                className="review-button"
                onClick={props.onRequestCleanup}
                disabled={!props.canReviewCleanup}
              >
                Cleanup
              </button>
            </div>
          </div>
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

    </div>
  );
}

type WorkspaceResultsProps = {
  artifacts: ArtifactAnalysis[];
  candidatesByPath: Map<string, CleanupCandidate>;
  selectedItemId: string | null;
  selectedPaths: string[];
  analysisReady: boolean;
  sort: WorkspaceSortState;
  onSort: (column: WorkspaceSortColumn) => void;
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
    <div className="workspace-table" role="table" aria-label="Workspace artifacts">
      <div className="result-row result-row-header" role="row">
        <span aria-hidden="true" />
        <SortableHeader
          label="Project"
          active={props.sort.column === 'project'}
          direction={props.sort.direction}
          onSort={() => props.onSort('project')}
        />
        <SortableHeader
          label="Artifact"
          active={props.sort.column === 'artifact'}
          direction={props.sort.direction}
          onSort={() => props.onSort('artifact')}
        />
        <SortableHeader
          label="Size"
          active={props.sort.column === 'size'}
          direction={props.sort.direction}
          onSort={() => props.onSort('size')}
        />
        <div className="modified-column">
          <SortableHeader
            label="Modified"
            active={props.sort.column === 'modified'}
            direction={props.sort.direction}
            onSort={() => props.onSort('modified')}
          />
        </div>
        <SortableHeader
          label="Status"
          active={props.sort.column === 'status'}
          direction={props.sort.direction}
          onSort={() => props.onSort('status')}
        />
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
