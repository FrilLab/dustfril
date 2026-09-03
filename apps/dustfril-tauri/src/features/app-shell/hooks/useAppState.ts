import { useEffect, useMemo, useRef, useState } from 'react';
import { formatBytes } from '../../../lib/format';
import {
  analyzeWorkspace,
  chooseWorkspaceFolder,
  defaultRoot,
  executeCleanup,
  loadActivityHistory,
} from '../../../lib/tauri';
import { categoryConfigs, type SidebarCategory } from '../../../model/categories';
import type { SidebarEntry } from '../../../components/Sidebar/Sidebar';
import type {
  ActivityRecord,
  AnalysisResponse,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
} from '../../../types/workflow';
import {
  cleanupAgeOptions,
  defaultCleanupAgeDays,
  deleteModes,
  ecosystems,
} from '../../../types/workflow';
import { createWorkspaceSummary, filterArtifacts } from '../../../model/presentation';

export function useAppState() {
  const [root, setRoot] = useState('');
  const [search, setSearch] = useState('');
  const [activeCategory, setActiveCategory] = useState<SidebarCategory>('workspace');
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [deleteMode, setDeleteMode] = useState<DeleteMode>('Trash');
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlanResponse | null>(null);
  const [cleanupResult, setCleanupResult] = useState<CleanupResultResponse | null>(null);
  const [historyEntries, setHistoryEntries] = useState<ActivityRecord[]>([]);
  const [selectedCleanupPaths, setSelectedCleanupPaths] = useState<string[]>([]);
  const [cleanupAgeDays, setCleanupAgeDays] = useState<number>(defaultCleanupAgeDays);
  const [confirmDialogOpen, setConfirmDialogOpen] = useState(false);
  const [lastAnalysisAtMs, setLastAnalysisAtMs] = useState<number | null>(null);
  const workspaceRequestRef = useRef(0);
  const actionRequestRef = useRef(0);

  useEffect(() => {
    defaultRoot()
      .then((path) => setRoot((current) => current || path))
      .catch((invokeError) => setError(String(invokeError)));

    loadActivityHistory()
      .then(setHistoryEntries)
      .catch((invokeError) => setError(String(invokeError)));
  }, []);

  const workspaceArtifacts = analysisResult?.artifacts ?? [];
  const cleanupCandidates = cleanupPlan?.candidates ?? [];
  const filteredArtifacts = useMemo(
    () => filterArtifacts(workspaceArtifacts, search),
    [workspaceArtifacts, search],
  );

  useEffect(() => {
    if (!selectedItemId) {
      return;
    }

    if (!filteredArtifacts.some((artifact) => artifact.path === selectedItemId)) {
      setSelectedItemId(null);
    }
  }, [filteredArtifacts, selectedItemId]);

  const selectedCandidateBytes = useMemo(
    () =>
      cleanupCandidates
        .filter((candidate) => selectedCleanupPaths.includes(candidate.path))
        .reduce((total, candidate) => total + candidate.sizeBytes, 0),
    [cleanupCandidates, selectedCleanupPaths],
  );

  const summary = useMemo(
    () => createWorkspaceSummary(workspaceArtifacts, cleanupPlan?.reclaimableSizeBytes ?? 0),
    [workspaceArtifacts, cleanupPlan?.reclaimableSizeBytes],
  );

  const discoveredEcosystems = useMemo(
    () =>
      ecosystems.filter((ecosystem) =>
        workspaceArtifacts.some((artifact) => artifact.ecosystem === ecosystem),
      ),
    [workspaceArtifacts],
  );

  const sidebarEntries = useMemo<SidebarEntry[]>(
    () =>
      categoryConfigs.map((config) => ({
        ...config,
        count:
          config.key === 'workspace'
            ? workspaceArtifacts.length
            : config.key === 'history'
              ? historyEntries.length
              : null,
      })),
    [historyEntries.length, workspaceArtifacts.length],
  );

  const statusMessage = error
    ? error
      : cleanupResult
      ? `Last cleanup freed ${formatBytes(cleanupResult.freedSizeBytes)} across ${cleanupResult.deletedPaths.length} path(s).`
      : analysisResult
        ? 'Review recommendations based on inactivity age. Trash is the default cleanup mode.'
        : 'Choose a workspace folder, then analyze it to find development artifacts.';

  const canAnalyze = busyAction === null && root.length > 0;
  const canReviewCleanup =
    busyAction === null && cleanupPlan !== null && selectedCleanupPaths.length > 0;
  const confirmSamplePaths = selectedCleanupPaths.slice(0, 5);

  async function runAction(action: string, runner: () => Promise<void>) {
    const requestId = ++actionRequestRef.current;
    setBusyAction(action);
    setError(null);

    try {
      await runner();
    } catch (invokeError) {
      if (requestId === actionRequestRef.current) {
        setError(String(invokeError));

        const refreshedHistory = await loadActivityHistory().catch(() => null);
        if (requestId === actionRequestRef.current && refreshedHistory) {
          setHistoryEntries(refreshedHistory);
        }
      }
    } finally {
      if (requestId === actionRequestRef.current) {
        setBusyAction(null);
      }
    }
  }

  function handleRootChange(nextRoot: string) {
    if (nextRoot === root) {
      return;
    }

    workspaceRequestRef.current += 1;
    actionRequestRef.current += 1;
    setRoot(nextRoot);
    setError(null);
    setAnalysisResult(null);
    setCleanupPlan(null);
    setCleanupResult(null);
    setSelectedCleanupPaths([]);
    setSelectedItemId(null);
    setConfirmDialogOpen(false);
    setLastAnalysisAtMs(null);
    setSearch('');
    setBusyAction(null);
  }

  async function handleChooseWorkspace() {
    if (busyAction !== null) {
      return;
    }

    try {
      const selected = await chooseWorkspaceFolder(root || undefined);
      if (selected) {
        handleRootChange(selected);
      }
    } catch (invokeError) {
      setError(String(invokeError));
    }
  }

  async function handleAnalyzeWorkspace() {
    if (!canAnalyze) {
      return;
    }

    await analyzeWorkspaceWithPolicy(cleanupAgeDays, true, true);
  }

  async function analyzeWorkspaceWithPolicy(
    policyAgeDays: number,
    recordHistory: boolean,
    recordArtifactSnapshot: boolean,
  ) {
    await runAction('analyze-workspace', async () => {
      const requestId = ++workspaceRequestRef.current;
      const response = await analyzeWorkspace({
        root,
        ecosystems: [...ecosystems],
        cleanupAgeDays: policyAgeDays,
        recordHistory,
        recordArtifactSnapshot,
      });

      if (requestId !== workspaceRequestRef.current) {
        return;
      }

      setAnalysisResult(response.analysis);
      setCleanupPlan(response.cleanupPlan);
      // Rebuild the default cleanup selection from the new policy. This
      // conservatively drops items that are no longer recommended and never
      // broadens the selection without a new recommendation.
      setSelectedCleanupPaths(
        response.cleanupPlan.candidates
          .filter((candidate) => candidate.selectedByDefault)
          .map((candidate) => candidate.path),
      );
      setSelectedItemId((current) =>
        current && response.analysis.artifacts.some((artifact) => artifact.path === current)
          ? current
          : null,
      );
      setCleanupResult(null);
      setCleanupAgeDays(policyAgeDays);
      setLastAnalysisAtMs(Date.now());
      setError(
        [response.analysis.historyWarning, response.artifactSnapshotWarning]
          .filter((warning): warning is string => Boolean(warning))
          .join(' ') || null,
      );
      setHistoryEntries(await loadActivityHistory());
      setActiveCategory('workspace');
    });
  }

  async function handleCleanupAgeChange(nextAgeDays: number) {
    if (!cleanupAgeOptions.includes(nextAgeDays as (typeof cleanupAgeOptions)[number])) {
      return;
    }

    if (nextAgeDays === cleanupAgeDays) {
      return;
    }

    if (!analysisResult) {
      setCleanupAgeDays(nextAgeDays);
      return;
    }

    await analyzeWorkspaceWithPolicy(nextAgeDays, false, false);
  }

  function toggleCleanupPath(path: string) {
    setSelectedCleanupPaths((current) =>
      current.includes(path) ? current.filter((value) => value !== path) : [...current, path],
    );
  }

  function handleRequestCleanup() {
    if (canReviewCleanup) {
      setConfirmDialogOpen(true);
    }
  }

  async function handleConfirmCleanup() {
    if (!cleanupPlan) {
      return;
    }

    const candidates = cleanupPlan.candidates.filter((candidate) =>
      selectedCleanupPaths.includes(candidate.path),
    );

    await runAction('cleanup-execute', async () => {
      const result = await executeCleanup(
        root,
        [...ecosystems],
        cleanupPlan.analysisId,
        candidates.map(({ path, ecosystem }) => ({ path, ecosystem })),
        deleteMode,
      );

      setCleanupResult(result);
      setError(
        result.failedPaths.length
          ? formatCleanupFailure(result, result.historyWarning)
          : result.historyWarning ?? null,
      );
      setCleanupPlan((current) =>
        current
          ? {
              ...current,
              candidates: current.candidates.filter(
                (candidate) => !result.deletedPaths.includes(candidate.path),
              ),
              reclaimableSizeBytes: current.candidates
                .filter(
                  (candidate) =>
                    candidate.selectedByDefault && !result.deletedPaths.includes(candidate.path),
                )
                .reduce((total, candidate) => total + candidate.sizeBytes, 0),
            }
          : current,
      );
      setAnalysisResult((current) =>
        current
          ? {
              ...current,
              artifacts: current.artifacts.filter(
                (artifact) => !result.deletedPaths.includes(artifact.path),
              ),
              totalSizeBytes: current.artifacts
                .filter((artifact) => !result.deletedPaths.includes(artifact.path))
                .reduce((total, artifact) => total + artifact.sizeBytes, 0),
            }
          : current,
      );
      setSelectedCleanupPaths((current) =>
        current.filter((path) => !result.deletedPaths.includes(path)),
      );
      setSelectedItemId(null);
      setConfirmDialogOpen(false);
      setHistoryEntries(await loadActivityHistory());
    });
  }

  return {
    root,
    search,
    activeCategory,
    selectedItemId,
    deleteMode,
    busyAction,
    error,
    analysisResult,
    cleanupPlan,
    selectedCleanupItems: cleanupCandidates.filter((candidate) =>
      selectedCleanupPaths.includes(candidate.path),
    ),
    selectedCleanupPaths,
    selectedCandidateBytes,
    cleanupAgeDays,
    sidebarEntries,
    filteredArtifacts,
    historyEntries,
    confirmDialogOpen,
    confirmSamplePaths,
    lastAnalysisAtMs,
    canAnalyze,
    canReviewCleanup,
    statusMessage,
    summary,
    discoveredEcosystems,
    deleteModes,
    setSearch,
    setActiveCategory,
    setSelectedItemId,
    setDeleteMode,
    setConfirmDialogOpen,
    toggleCleanupPath,
    handleRootChange,
    handleChooseWorkspace,
    handleAnalyzeWorkspace,
    handleCleanupAgeChange,
    handleRequestCleanup,
    handleConfirmCleanup,
  };
}

function formatCleanupFailure(result: CleanupResultResponse, historyWarning?: string): string {
  const failures = result.failedPaths
    .map((failure) => `${failure.path} (${failure.reason})`)
    .join('; ');
  const historyMessage = historyWarning ? ` ${historyWarning}` : '';

  return `Cleanup completed with ${result.failedPaths.length} failure(s). Freed ${formatBytes(result.freedSizeBytes)}. Failed: ${failures}.${historyMessage}`;
}
