import { useEffect, useMemo, useRef, useState } from 'react';
import { formatBytes } from '../../../lib/format';
import {
  analyzeArtifacts,
  buildCleanupPlan,
  defaultRoot,
  executeCleanup,
  loadActivityHistory,
  scanArtifacts,
} from '../../../lib/tauri';
import {
  categoryConfigs,
  ecosystemForCategory,
  isFutureCategory,
  isLanguageCategory,
  type SidebarCategory,
} from '../../../model/categories';
import type { SidebarEntry } from '../../../components/Sidebar/Sidebar';
import type {
  AnalysisResponse,
  ActivityRecord,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  Ecosystem,
  RunOptions,
  ScanResponse,
} from '../../../types/workflow';
import { deleteModes, ecosystems } from '../../../types/workflow';
import {
  createAnalysisItems,
  createCleanupItems,
  createScanItems,
  createWorkspaceSummary,
  filterItems,
} from '../../../model/presentation';
import type { ExplorerWorkflow } from '../../../model/types';

export function useAppState() {
  const [root, setRoot] = useState('');
  const [search, setSearch] = useState('');
  const [activeCategory, setActiveCategory] = useState<SidebarCategory>('overview');
  const [explorerWorkflow, setExplorerWorkflow] = useState<ExplorerWorkflow>('scan');
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [deleteMode, setDeleteMode] = useState<DeleteMode>('Trash');
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResponse | null>(null);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlanResponse | null>(null);
  const [cleanupResult, setCleanupResult] = useState<CleanupResultResponse | null>(null);
  const [historyEntries, setHistoryEntries] = useState<ActivityRecord[]>([]);
  const [selectedCleanupPaths, setSelectedCleanupPaths] = useState<string[]>([]);
  const [confirmDialogOpen, setConfirmDialogOpen] = useState(false);
  const [lastScanAtMs, setLastScanAtMs] = useState<number | null>(null);
  const previousRootRef = useRef<string | null>(null);
  const workspaceRequestRef = useRef(0);
  const actionRequestRef = useRef(0);

  useEffect(() => {
    defaultRoot()
      .then(handleRootChange)
      .catch((invokeError) => setError(String(invokeError)));

    loadActivityHistory()
      .then(setHistoryEntries)
      .catch((invokeError) => setError(String(invokeError)));
  }, []);

  const activeEcosystem = ecosystemForCategory(activeCategory);

  const runOptions = useMemo<RunOptions>(() => {
    if (activeEcosystem) {
      return {
        root,
        ecosystems: [activeEcosystem],
      };
    }

    return {
      root,
      ecosystems: [...ecosystems],
    };
  }, [root, activeEcosystem]);

  useEffect(() => {
    if (!root) {
      return;
    }

    if (previousRootRef.current === root) {
      return;
    }

    previousRootRef.current = root;

    void runAction('bootstrap', async () => {
      await refreshWorkspaceData({
        root,
        ecosystems: [...ecosystems],
      });
    });
  }, [root]);

  const summary = useMemo(
    () => createWorkspaceSummary(analysisResult?.artifacts),
    [analysisResult?.artifacts],
  );

  const filteredScanArtifacts = useMemo(() => {
    const artifacts = scanResult?.artifacts ?? [];

    if (!activeEcosystem) {
      return artifacts;
    }

    return artifacts.filter((artifact) => artifact.ecosystem === activeEcosystem);
  }, [scanResult?.artifacts, activeEcosystem]);

  const filteredAnalysisArtifacts = useMemo(() => {
    const artifacts = analysisResult?.artifacts ?? [];

    if (!activeEcosystem) {
      return artifacts;
    }

    return artifacts.filter((artifact) => artifact.ecosystem === activeEcosystem);
  }, [analysisResult?.artifacts, activeEcosystem]);

  const filteredCleanupCandidates = useMemo(() => {
    const candidates = cleanupPlan?.candidates ?? [];

    if (!activeEcosystem) {
      return candidates;
    }

    return candidates.filter((candidate) => candidate.ecosystem === activeEcosystem);
  }, [cleanupPlan?.candidates, activeEcosystem]);

  const scanItems = useMemo(
    () => filterItems(createScanItems(filteredScanArtifacts), search),
    [filteredScanArtifacts, search],
  );
  const analysisItems = useMemo(
    () => filterItems(createAnalysisItems(filteredAnalysisArtifacts), search),
    [filteredAnalysisArtifacts, search],
  );
  const cleanupItems = useMemo(
    () =>
      filterItems(
        createCleanupItems(filteredCleanupCandidates, selectedCleanupPaths, deleteMode),
        search,
      ),
    [filteredCleanupCandidates, selectedCleanupPaths, deleteMode, search],
  );

  const explorerItems = useMemo(() => {
    if (explorerWorkflow === 'analysis') {
      return analysisItems;
    }

    if (explorerWorkflow === 'cleanup') {
      return cleanupItems;
    }

    return scanItems;
  }, [explorerWorkflow, analysisItems, cleanupItems, scanItems]);

  useEffect(() => {
    if (!explorerItems.length) {
      setSelectedItemId(null);
      return;
    }

    if (!selectedItemId || !explorerItems.some((item) => item.id === selectedItemId)) {
      setSelectedItemId(explorerItems[0].id);
    }
  }, [explorerItems, selectedItemId]);

  const selectedCandidateBytes = useMemo(() => {
    return filteredCleanupCandidates
      .filter((candidate) => selectedCleanupPaths.includes(candidate.path))
      .reduce((total, candidate) => total + candidate.sizeBytes, 0);
  }, [filteredCleanupCandidates, selectedCleanupPaths]);

  const sidebarEntries = useMemo<SidebarEntry[]>(() => {
    const scanArtifactsByEcosystem = (ecosystem: Ecosystem) =>
      (scanResult?.artifacts ?? []).filter((artifact) => artifact.ecosystem === ecosystem).length;

    return categoryConfigs.map((config) => {
      if (config.key === 'overview') {
        return {
          ...config,
          count: scanResult?.artifacts.length ?? 0,
        };
      }

      if (config.key === 'history') {
        return {
          ...config,
          count: historyEntries.length,
        };
      }

      if (config.ecosystem) {
        return {
          ...config,
          count: scanArtifactsByEcosystem(config.ecosystem),
        };
      }

      return {
        ...config,
        count: 0,
      };
    });
  }, [historyEntries.length, scanResult?.artifacts]);

  const activeCategoryConfig =
    categoryConfigs.find((config) => config.key === activeCategory) ?? categoryConfigs[0];

  const canRunActions = busyAction === null && root.length > 0;

  const statusMessage = error
    ? error
    : cleanupResult
      ? `Last cleanup freed ${formatBytes(cleanupResult.freedSizeBytes)} across ${cleanupResult.deletedPaths.length} paths.`
      : 'Select a location, scan the workspace, review artifacts, and confirm cleanup before files are moved to Trash.';

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

        // Failed operations can still leave an activity record behind. Refresh
        // it without allowing a history-read failure to hide the primary error.
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

  async function refreshWorkspaceData(options: RunOptions) {
    const requestId = ++workspaceRequestRef.current;
    const [scan, analysis, plan] = await Promise.all([
      scanArtifacts(options),
      analyzeArtifacts(options),
      buildCleanupPlan(options),
    ]);

    if (requestId !== workspaceRequestRef.current) {
      return;
    }

    setScanResult(scan);
    setAnalysisResult(analysis);
    setCleanupPlan(plan);
    setSelectedCleanupPaths(plan.candidates.map((candidate) => candidate.path));
    setLastScanAtMs(Date.now());
    setError(formatScanWarnings(scan));
    setHistoryEntries(await loadActivityHistory());
  }

  function handleRootChange(nextRoot: string) {
    if (nextRoot === root) {
      return;
    }

    workspaceRequestRef.current += 1;
    setRoot(nextRoot);
    setError(null);
    setScanResult(null);
    setAnalysisResult(null);
    setCleanupPlan(null);
    setCleanupResult(null);
    setSelectedCleanupPaths([]);
    setSelectedItemId(null);
    setConfirmDialogOpen(false);
    setLastScanAtMs(null);
    setExplorerWorkflow('scan');
  }

  function toggleCleanupPath(path: string) {
    setSelectedCleanupPaths((current) =>
      current.includes(path) ? current.filter((value) => value !== path) : [...current, path],
    );
  }

  async function handleScanCategory() {
    await runAction('scan', async () => {
      await refreshWorkspaceData(runOptions);
      setExplorerWorkflow('scan');
    });
  }

  async function handleAnalyzeCategory() {
    await runAction('analyze', async () => {
      setAnalysisResult(await analyzeArtifacts(runOptions));
      setHistoryEntries(await loadActivityHistory());
      setExplorerWorkflow('analysis');
    });
  }

  async function handleBuildCleanupPlan() {
    await runAction('cleanup-plan', async () => {
      const plan = await buildCleanupPlan(runOptions);
      setCleanupPlan(plan);
      setSelectedCleanupPaths(plan.candidates.map((candidate) => candidate.path));
      setHistoryEntries(await loadActivityHistory());
      setExplorerWorkflow('cleanup');
    });
  }

  function handleRequestCleanup() {
    if (!selectedCleanupPaths.length) {
      return;
    }

    setConfirmDialogOpen(true);
  }

  async function handleConfirmCleanup() {
    if (!cleanupPlan) {
      return;
    }

    const candidates = cleanupPlan.candidates.filter((candidate) =>
      selectedCleanupPaths.includes(candidate.path),
    );

    await runAction('cleanup-execute', async () => {
      const result = await executeCleanup(candidates, deleteMode);

      setCleanupResult(result);
      if (result.failedPaths.length) {
        setError(formatCleanupFailure(result, result.historyWarning));
      } else {
        setError(result.historyWarning ?? null);
      }
      setCleanupPlan((current) =>
        current
          ? {
              ...current,
              candidates: current.candidates.filter(
                (candidate) => !result.deletedPaths.includes(candidate.path),
              ),
              reclaimableSizeBytes: current.candidates
                .filter((candidate) => !result.deletedPaths.includes(candidate.path))
                .reduce((total, candidate) => total + candidate.sizeBytes, 0),
            }
          : current,
      );
      setSelectedCleanupPaths((current) =>
        current.filter((path) => !result.deletedPaths.includes(path)),
      );
      setConfirmDialogOpen(false);
      setHistoryEntries(await loadActivityHistory());
    });
  }

  return {
    root,
    search,
    activeCategory,
    activeCategoryConfig,
    explorerWorkflow,
    selectedItemId,
    deleteMode,
    busyAction,
    error,
    selectedCleanupPaths,
    selectedCandidateBytes,
    sidebarEntries,
    scanItems,
    analysisItems,
    cleanupItems,
    explorerItems,
    historyEntries,
    confirmDialogOpen,
    confirmSamplePaths,
    lastScanAtMs,
    canRunActions,
    statusMessage,
    reclaimableBytes: cleanupPlan?.reclaimableSizeBytes ?? 0,
    artifactCount: scanResult?.artifacts.length ?? 0,
    summary,
    deleteModes,
    supportedEcosystems: ecosystems,
    isLanguageCategory,
    isFutureCategory,
    setRoot: handleRootChange,
    setSearch,
    setActiveCategory,
    setExplorerWorkflow,
    setSelectedItemId,
    setDeleteMode,
    setConfirmDialogOpen,
    toggleCleanupPath,
    handleScanCategory,
    handleAnalyzeCategory,
    handleBuildCleanupPlan,
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

function formatScanWarnings(
  scan: Pick<ScanResponse, 'historyWarning' | 'artifactSnapshotWarning'>,
): string | null {
  const warnings = [scan.historyWarning, scan.artifactSnapshotWarning].filter(
    (warning): warning is string => Boolean(warning),
  );

  return warnings.length ? warnings.join(' ') : null;
}
