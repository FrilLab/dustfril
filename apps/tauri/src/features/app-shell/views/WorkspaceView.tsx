import { useEffect, useMemo, useState } from 'react';
import { AsyncStatePanel } from '../../../components/AsyncStatePanel/AsyncStatePanel';
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
  selectedCandidateBytes,
} from '../../../model/presentation';
import type {
  ArtifactAnalysis,
  CleanupCandidate,
  DeleteMode,
  Ecosystem,
} from '../../../types/workflow';
import type { AsyncOperationStatus } from '../../../model/async';
import { cleanupAgeOptions } from '../../../types/workflow';
import {
  sortArtifacts,
  type WorkspaceSortColumn,
  type WorkspaceSortState,
} from '../../../model/sorting';

type WorkspaceViewProps = {
  ecosystem?: Ecosystem;
  artifacts: ArtifactAnalysis[];
  candidates: CleanupCandidate[];
  operationStatus?: AsyncOperationStatus;
  selectedItemId: string | null;
  selectedPaths: string[];
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
  onRequestCleanup: (paths: string[]) => void;
};

export function WorkspaceView(props: WorkspaceViewProps) {
  const [sort, setSort] = useState<WorkspaceSortState>({ column: 'size', direction: 'desc' });
  const visibleArtifacts = useMemo(
    () => props.ecosystem ? props.artifacts.filter((artifact) => artifact.ecosystem === props.ecosystem) : props.artifacts,
    [props.artifacts, props.ecosystem],
  );
  const visibleCandidates = useMemo(
    () => props.ecosystem ? props.candidates.filter((candidate) => candidate.ecosystem === props.ecosystem) : props.candidates,
    [props.candidates, props.ecosystem],
  );
  const visibleSelectedPaths = useMemo(
    () => props.selectedPaths.filter((path) => visibleCandidates.some((candidate) => candidate.path === path)),
    [props.selectedPaths, visibleCandidates],
  );
  const visibleSelectedBytes = useMemo(
    () => selectedCandidateBytes(visibleCandidates, visibleSelectedPaths),
    [visibleCandidates, visibleSelectedPaths],
  );
  const candidatesByPath = new Map(visibleCandidates.map((candidate) => [candidate.path, candidate]));
  const selectedArtifact =
    visibleArtifacts.find((artifact) => artifact.path === props.selectedItemId) ?? null;
  const sortedArtifacts = useMemo(
    () => sortArtifacts(visibleArtifacts, sort),
    [visibleArtifacts, sort],
  );
  const visibleTotalSizeBytes = visibleArtifacts.reduce(
    (total, artifact) => total + artifact.sizeBytes,
    0,
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
            {formatCount(visibleArtifacts.length)} artifact{visibleArtifacts.length === 1 ? '' : 's'}
          </strong>{' '}
          ·{' '}
          <strong>{formatBytes(visibleTotalSizeBytes)}</strong> total
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
            selectedPaths={visibleSelectedPaths}
            analysisReady={props.analysisReady}
            operationStatus={props.operationStatus ?? 'idle'}
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
                {formatCount(visibleSelectedPaths.length)} selected · {formatBytes(visibleSelectedBytes)}
              </span>
              <button
                type="button"
                className="review-button"
                onClick={() => props.onRequestCleanup(visibleSelectedPaths)}
                disabled={!props.canReviewCleanup || visibleSelectedPaths.length === 0}
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
            selectedPaths={visibleSelectedPaths}
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
  operationStatus: AsyncOperationStatus;
  sort: WorkspaceSortState;
  onSort: (column: WorkspaceSortColumn) => void;
  onSelectItem: (path: string) => void;
  onTogglePath: (path: string) => void;
};

function WorkspaceResults(props: WorkspaceResultsProps) {
  if (props.operationStatus === 'loading' && !props.analysisReady) {
    return (
      <AsyncStatePanel
        status="loading"
        title="Analyzing workspace"
        description="DustFril is scanning the selected workspace for supported development artifacts."
      />
    );
  }

  if (props.operationStatus === 'error' && !props.analysisReady) {
    return (
      <AsyncStatePanel
        status="error"
        title="Workspace analysis failed"
        description="The workspace could not be analyzed. Choose another workspace or try again."
      />
    );
  }

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
