import { useEffect, useMemo, useReducer, useRef, useState } from 'react';
import { formatBytes } from '../../../lib/format';
import {
  analyzeWorkspace,
  chooseWorkspaceFolder,
  clearActivityHistory,
  defaultRoot,
  executeCleanup,
  loadActivityHistory,
  refreshStorageVolume,
  workflowSecurityScan,
} from '../../../lib/tauri';
import { categoryConfigs, type SidebarCategory } from '../../../model/categories';
import {
  idleAsyncOperation,
  reduceAsyncOperation,
} from '../../../model/async';
import type { SidebarEntry } from '../../../components/Sidebar/Sidebar';
import type {
  ActivityRecord,
  AnalysisResponse,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  StorageSummary,
  VolumeStorage,
  WorkflowScanResponse,
} from '../../../types/workflow';
import { cleanupAgeOptions, defaultCleanupAgeDays, deleteModes, ecosystems } from '../../../types/workflow';
import {
  createWorkspaceSummary,
  filterArtifacts,
  selectedCandidateBytes,
} from '../../../model/presentation';

type WorkspaceAnalysisResponse = Awaited<ReturnType<typeof analyzeWorkspace>>;

export function useAppState() {
  const [root, setRoot] = useState('');
  const [search, setSearch] = useState('');
  const [activeCategory, setActiveCategory] = useState<SidebarCategory>('cleanup-rust');
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [deleteMode, setDeleteMode] = useState<DeleteMode>('Trash');
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlanResponse | null>(null);
  const [storageSummary, setStorageSummary] = useState<StorageSummary | null>(null);
  const [historyEntries, setHistoryEntries] = useState<ActivityRecord[]>([]);
  const [selectedCleanupPaths, setSelectedCleanupPaths] = useState<string[]>([]);
  const [cleanupReviewPaths, setCleanupReviewPaths] = useState<string[]>([]);
  const [cleanupAgeDays, setCleanupAgeDays] = useState<number>(defaultCleanupAgeDays);
  const [confirmDialogOpen, setConfirmDialogOpen] = useState(false);
  const [workspaceOperation, dispatchWorkspaceOperation] = useReducer(
    reduceAsyncOperation<WorkspaceAnalysisResponse>,
    idleAsyncOperation<WorkspaceAnalysisResponse>(),
  );
  const [workflowOperation, dispatchWorkflowOperation] = useReducer(
    reduceAsyncOperation<WorkflowScanResponse>,
    idleAsyncOperation<WorkflowScanResponse>(),
  );
  const workspaceRequestRef = useRef(0);
  const workflowRequestRef = useRef(0);
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

  const cleanupReviewItems = useMemo(
    () => cleanupCandidates.filter((candidate) => cleanupReviewPaths.includes(candidate.path)),
    [cleanupCandidates, cleanupReviewPaths],
  );
  const cleanupReviewTotalBytes = useMemo(
    () => selectedCandidateBytes(cleanupCandidates, cleanupReviewPaths),
    [cleanupCandidates, cleanupReviewPaths],
  );

  const summary = useMemo(
    () => createWorkspaceSummary(workspaceArtifacts, cleanupPlan?.reclaimableSizeBytes ?? 0),
    [workspaceArtifacts, cleanupPlan?.reclaimableSizeBytes],
  );

  const sidebarEntries = useMemo<SidebarEntry[]>(
    () =>
      categoryConfigs.map((config) => ({
        ...config,
        count:
          config.ecosystem
            ? workspaceArtifacts.filter((artifact) => artifact.ecosystem === config.ecosystem).length
            : config.key === 'workspace-activity' || config.key === 'history'
              ? historyEntries.length
              : null,
      })),
    [historyEntries.length, workspaceArtifacts],
  );

  const canAnalyze = busyAction === null && root.length > 0;
  const canScanWorkflows = busyAction === null && root.length > 0;
  const canReviewCleanup =
    busyAction === null && cleanupPlan !== null && selectedCleanupPaths.length > 0;
  const confirmSamplePaths = cleanupReviewPaths.slice(0, 5);

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
    dispatchWorkspaceOperation({
      type: 'invalidate',
      requestId: workspaceRequestRef.current,
    });
    workflowRequestRef.current += 1;
    dispatchWorkflowOperation({
      type: 'invalidate',
      requestId: workflowRequestRef.current,
    });
    actionRequestRef.current += 1;
    setRoot(nextRoot);
    setError(null);
    setAnalysisResult(null);
    setCleanupPlan(null);
    setStorageSummary(null);
    setSelectedCleanupPaths([]);
    setCleanupReviewPaths([]);
    setSelectedItemId(null);
    setConfirmDialogOpen(false);
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

  async function handleWorkflowSecurityScan() {
    if (busyAction !== null || !root) {
      return;
    }

    await runAction('workflow-security-scan', async () => {
      const requestId = ++workflowRequestRef.current;
      dispatchWorkflowOperation({ type: 'start', requestId });

      try {
        const response = await workflowSecurityScan({
          root,
          ecosystems: [],
        });

        dispatchWorkflowOperation({
          type: 'success',
          requestId,
          data: response,
          warnings: response.notices.map((notice) => notice.reason),
        });
      } catch (invokeError) {
        dispatchWorkflowOperation({
          type: 'error',
          requestId,
          error: String(invokeError),
        });
        throw invokeError;
      }
    });
  }

  async function analyzeWorkspaceWithPolicy(
    policyAgeDays: number,
    recordHistory: boolean,
    recordArtifactSnapshot: boolean,
  ) {
    await runAction('analyze-workspace', async () => {
      const requestId = ++workspaceRequestRef.current;
      dispatchWorkspaceOperation({ type: 'start', requestId });

      try {
        const response = await analyzeWorkspace({
          root,
          ecosystems: [...ecosystems],
          cleanupAgeDays: policyAgeDays,
          recordHistory,
          recordArtifactSnapshot,
        });

        dispatchWorkspaceOperation({
          type: 'success',
          requestId,
          data: response,
          warnings: [response.analysis.historyWarning, response.artifactSnapshotWarning].filter(
            (warning): warning is string => Boolean(warning),
          ),
        });

        if (requestId !== workspaceRequestRef.current) {
          return;
        }

        setAnalysisResult(response.analysis);
        setCleanupPlan(response.cleanupPlan);
        setStorageSummary(response.storageSummary);
        // Rebuild the default cleanup selection from the new policy. This
        // conservatively drops items that are no longer recommended and never
        // broadens the selection without a new recommendation.
        setSelectedCleanupPaths(
          response.cleanupPlan.candidates
            .filter((candidate) => candidate.selectedByDefault)
            .map((candidate) => candidate.path),
        );
        setCleanupReviewPaths([]);
        setSelectedItemId((current) =>
          current && response.analysis.artifacts.some((artifact) => artifact.path === current)
            ? current
            : null,
        );
        setCleanupAgeDays(policyAgeDays);
        setError(
          [response.analysis.historyWarning, response.artifactSnapshotWarning]
            .filter((warning): warning is string => Boolean(warning))
            .join(' ') || null,
        );
        const refreshedHistory = await loadActivityHistory();
        if (requestId !== workspaceRequestRef.current) {
          return;
        }
        setHistoryEntries(refreshedHistory);
        setActiveCategory((current) => (current === 'overview' ? 'cleanup-rust' : current));
      } catch (invokeError) {
        dispatchWorkspaceOperation({
          type: 'error',
          requestId,
          error: String(invokeError),
        });
        throw invokeError;
      }
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

  async function handleClearHistory() {
    if (busyAction !== null) {
      return;
    }

    const requestId = ++actionRequestRef.current;
    setBusyAction('history-clear');
    setError(null);

    try {
      await clearActivityHistory();
      if (requestId === actionRequestRef.current) {
        setHistoryEntries([]);
      }
    } catch (invokeError) {
      if (requestId === actionRequestRef.current) {
        setError(String(invokeError));
      }
      throw invokeError;
    } finally {
      if (requestId === actionRequestRef.current) {
        setBusyAction(null);
      }
    }
  }

  function toggleCleanupPath(path: string) {
    setSelectedCleanupPaths((current) =>
      current.includes(path) ? current.filter((value) => value !== path) : [...current, path],
    );
  }

  function openWorkspaceArtifact(path: string) {
    setSearch('');
    setSelectedItemId(path);
    const artifact = analysisResult?.artifacts.find((candidate) => candidate.path === path);
    setActiveCategory(categoryForEcosystem(artifact?.ecosystem));
  }

  function handleRequestCleanup(paths: string[]) {
    if (busyAction !== null || cleanupPlan === null) {
      return;
    }

    const candidatePaths = new Set(cleanupPlan.candidates.map((candidate) => candidate.path));
    const reviewPaths = [...new Set(paths)].filter((path) => candidatePaths.has(path));
    if (!reviewPaths.length) {
      return;
    }

    setCleanupReviewPaths(reviewPaths);
    setConfirmDialogOpen(true);
  }

  async function handleConfirmCleanup() {
    if (!cleanupPlan) {
      return;
    }

    const candidates = cleanupPlan.candidates.filter((candidate) =>
      cleanupReviewPaths.includes(candidate.path),
    );

    await runAction('cleanup-execute', async () => {
      const result = await executeCleanup(
        root,
        [...ecosystems],
        cleanupPlan.analysisId,
        candidates.map(({ path, ecosystem }) => ({ path, ecosystem })),
        deleteMode,
      );

      let refreshedVolume: VolumeStorage | null = null;
      let storageRefreshWarning: string | null = null;
      if (deleteMode === 'Permanent' && result.deletedPaths.length > 0) {
        try {
          refreshedVolume = await refreshStorageVolume(root);
        } catch (invokeError) {
          storageRefreshWarning = `Failed to refresh storage statistics: ${String(invokeError)}`;
        }
      }

      setError(
        [
          result.failedPaths.length
            ? formatCleanupFailure(result, result.historyWarning)
            : result.historyWarning,
          storageRefreshWarning,
        ]
          .filter((warning): warning is string => Boolean(warning))
          .join(' ') || null,
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
      const deletedCandidates = candidates.filter((candidate) =>
        result.deletedPaths.includes(candidate.path),
      );
      const deletedDetectedBytes = deletedCandidates.reduce(
        (total, candidate) => total + candidate.sizeBytes,
        0,
      );
      const deletedRecommendedBytes = deletedCandidates
        .filter((candidate) => candidate.selectedByDefault)
        .reduce((total, candidate) => total + candidate.sizeBytes, 0);
      setStorageSummary((current) =>
        current?.status === 'available'
          ? (() => {
              const nextDetectedBytes = Math.max(
                0,
                current.detectedDevelopmentBytes - deletedDetectedBytes,
              );
              const nextVolume = refreshedVolume ?? {
                totalBytes: current.totalBytes,
                usedBytes: current.usedBytes,
                availableBytes: current.availableBytes,
              };

              return {
                ...current,
                ...nextVolume,
                detectedDevelopmentBytes: nextDetectedBytes,
                detectedSharePercent:
                  nextVolume.usedBytes > 0
                    ? (nextDetectedBytes / nextVolume.usedBytes) * 100
                    : null,
                recommendedBytes: Math.max(0, current.recommendedBytes - deletedRecommendedBytes),
              };
            })()
          : current,
      );
      setSelectedCleanupPaths((current) =>
        current.filter((path) => !result.deletedPaths.includes(path)),
      );
      setCleanupReviewPaths([]);
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
    storageSummary,
    selectedCleanupItems: cleanupReviewItems,
    selectedCleanupPaths,
    selectedCandidateBytes: cleanupReviewTotalBytes,
    cleanupAgeDays,
    sidebarEntries,
    filteredArtifacts,
    historyEntries,
    confirmDialogOpen,
    confirmSamplePaths,
    workspaceOperation,
    workflowOperation,
    canAnalyze,
    canScanWorkflows,
    canReviewCleanup,
    summary,
    deleteModes,
    setSearch,
    setActiveCategory,
    setSelectedItemId,
    setDeleteMode,
    setConfirmDialogOpen,
    toggleCleanupPath,
    openWorkspaceArtifact,
    handleRootChange,
    handleChooseWorkspace,
    handleAnalyzeWorkspace,
    handleWorkflowSecurityScan,
    handleCleanupAgeChange,
    handleClearHistory,
    handleRequestCleanup,
    handleConfirmCleanup,
  };
}

function categoryForEcosystem(
  ecosystem: WorkspaceAnalysisResponse['analysis']['artifacts'][number]['ecosystem'] | undefined,
): SidebarCategory {
  switch (ecosystem) {
    case 'Node':
      return 'cleanup-node';
    case 'Java':
      return 'cleanup-java';
    case 'Rust':
    default:
      return 'cleanup-rust';
  }
}

function formatCleanupFailure(result: CleanupResultResponse, historyWarning?: string): string {
  const failures = result.failedPaths
    .map((failure) => `${failure.path} (${failure.reason})`)
    .join('; ');
  const historyMessage = historyWarning ? ` ${historyWarning}` : '';

  return `Cleanup completed with ${result.failedPaths.length} failure(s). Freed ${formatBytes(result.freedSizeBytes)}. Failed: ${failures}.${historyMessage}`;
}
