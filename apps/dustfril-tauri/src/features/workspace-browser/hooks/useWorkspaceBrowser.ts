import { useEffect, useMemo, useRef, useState } from 'react';
import { formatBytes } from '../../../lib/format';
import {
  analyzeArtifacts,
  auditScripts,
  buildCleanupPlan,
  defaultRoot,
  executeCleanup,
  scanArtifacts,
} from '../../../lib/tauri';
import type {
  AnalysisResponse,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  Ecosystem,
  LifecycleScript,
  RunOptions,
  ScanResponse,
} from '../../../types/workflow';
import { deleteModes, ecosystems } from '../../../types/workflow';
import {
  createAnalysisItems,
  createAuditItems,
  createCleanupItems,
  createFooterStats,
  createPaneConfigs,
  createScanItems,
  createStatusMetrics,
  createTotalsMetrics,
  createWorkspaceSummary,
  filterItems,
  primaryActionLabel,
} from '../model/presentation';
import type { BrowserPane } from '../model/types';

export function useWorkspaceBrowser() {
  const [root, setRoot] = useState('');
  const [search, setSearch] = useState('');
  const [activePane, setActivePane] = useState<BrowserPane>('analysis');
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [selectedEcosystems, setSelectedEcosystems] = useState<Ecosystem[]>(ecosystems);
  const [deleteMode, setDeleteMode] = useState<DeleteMode>('Trash');
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResponse | null>(null);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlanResponse | null>(null);
  const [cleanupResult, setCleanupResult] = useState<CleanupResultResponse | null>(null);
  const [auditResult, setAuditResult] = useState<LifecycleScript[]>([]);
  const [selectedCleanupPaths, setSelectedCleanupPaths] = useState<string[]>([]);
  const initializedRef = useRef(false);

  useEffect(() => {
    defaultRoot().then(setRoot).catch((invokeError) => setError(String(invokeError)));
  }, []);

  const runOptions = useMemo<RunOptions>(
    () => ({
      root,
      ecosystems: selectedEcosystems,
    }),
    [root, selectedEcosystems],
  );

  useEffect(() => {
    if (!root || initializedRef.current) {
      return;
    }

    initializedRef.current = true;

    void runAction('bootstrap', async () => {
      const [scan, analysis, plan] = await Promise.all([
        scanArtifacts({ root, ecosystems: selectedEcosystems }),
        analyzeArtifacts({ root, ecosystems: selectedEcosystems }),
        buildCleanupPlan({ root, ecosystems: selectedEcosystems }),
      ]);

      setScanResult(scan);
      setAnalysisResult(analysis);
      setCleanupPlan(plan);
      setSelectedCleanupPaths(plan.candidates.map((candidate) => candidate.path));
    });
  }, [root, selectedEcosystems]);

  const summary = useMemo(
    () => createWorkspaceSummary(analysisResult?.artifacts),
    [analysisResult?.artifacts],
  );

  const selectedCandidateCount = selectedCleanupPaths.length;
  const selectedCandidateBytes = useMemo(() => {
    const candidates = cleanupPlan?.candidates ?? [];

    return candidates
      .filter((candidate) => selectedCleanupPaths.includes(candidate.path))
      .reduce((total, candidate) => total + candidate.sizeBytes, 0);
  }, [cleanupPlan?.candidates, selectedCleanupPaths]);

  const paneConfigs = useMemo(
    () =>
      createPaneConfigs({
        analysisCount: analysisResult?.artifacts.length ?? 0,
        cleanupCount: cleanupPlan?.candidates.length ?? 0,
        scanCount: scanResult?.artifacts.length ?? 0,
        auditCount: auditResult.length,
      }),
    [analysisResult?.artifacts.length, cleanupPlan?.candidates.length, scanResult?.artifacts.length, auditResult.length],
  );

  const analysisItems = useMemo(
    () => createAnalysisItems(analysisResult?.artifacts),
    [analysisResult?.artifacts],
  );
  const cleanupItems = useMemo(
    () => createCleanupItems(cleanupPlan?.candidates, selectedCleanupPaths, deleteMode),
    [cleanupPlan?.candidates, selectedCleanupPaths, deleteMode],
  );
  const scanItems = useMemo(() => createScanItems(scanResult?.artifacts), [scanResult?.artifacts]);
  const auditItems = useMemo(() => createAuditItems(auditResult), [auditResult]);

  const currentItems = useMemo(() => {
    const itemsByPane = {
      analysis: analysisItems,
      cleanup: cleanupItems,
      scan: scanItems,
      audit: auditItems,
    };

    return filterItems(itemsByPane[activePane], search);
  }, [activePane, analysisItems, cleanupItems, scanItems, auditItems, search]);

  useEffect(() => {
    if (!currentItems.length) {
      setSelectedItemId(null);
      return;
    }

    if (!selectedItemId || !currentItems.some((item) => item.id === selectedItemId)) {
      setSelectedItemId(currentItems[0].id);
    }
  }, [currentItems, selectedItemId]);

  const selectedItem = currentItems.find((item) => item.id === selectedItemId) ?? null;
  const activePaneConfig = paneConfigs.find((pane) => pane.key === activePane) ?? paneConfigs[0];
  const canRunActions = busyAction === null && selectedEcosystems.length > 0;

  const statusMetrics = useMemo(
    () =>
      createStatusMetrics({
        summary,
        selectedCandidateCount,
        selectedCandidateBytes,
        auditCount: auditResult.length,
      }),
    [summary, selectedCandidateCount, selectedCandidateBytes, auditResult.length],
  );

  const totalsMetrics = useMemo(
    () =>
      createTotalsMetrics({
        analyzedSizeBytes: analysisResult?.totalSizeBytes ?? 0,
        safeCount: summary.safeCount,
        safeBytes: summary.safeBytes,
        cleanupPoolBytes: cleanupPlan?.reclaimableSizeBytes ?? 0,
        auditCount: auditResult.length,
      }),
    [
      analysisResult?.totalSizeBytes,
      summary.safeCount,
      summary.safeBytes,
      cleanupPlan?.reclaimableSizeBytes,
      auditResult.length,
    ],
  );

  const footerStats = useMemo(
    () =>
      createFooterStats({
        keepCount: summary.keepCount,
        reviewCount: summary.reviewCount,
        cleanupCount: cleanupPlan?.candidates.length ?? 0,
        auditCount: auditResult.length,
      }),
    [summary.keepCount, summary.reviewCount, cleanupPlan?.candidates.length, auditResult.length],
  );

  const statusMessage = error
    ? error
    : cleanupResult
      ? `Last cleanup freed ${formatBytes(cleanupResult.freedSizeBytes)} across ${cleanupResult.deletedPaths.length} paths.`
      : 'Workspace loaded. Use the left sidebar to rescan, analyze, queue cleanup, or audit scripts.';

  async function runAction(action: string, runner: () => Promise<void>) {
    setBusyAction(action);
    setError(null);

    try {
      await runner();
    } catch (invokeError) {
      setError(String(invokeError));
    } finally {
      setBusyAction(null);
    }
  }

  function toggleEcosystem(ecosystem: Ecosystem) {
    setSelectedEcosystems((current) =>
      current.includes(ecosystem)
        ? current.filter((value) => value !== ecosystem)
        : [...current, ecosystem],
    );
  }

  function toggleCleanupPath(path: string) {
    setSelectedCleanupPaths((current) =>
      current.includes(path) ? current.filter((value) => value !== path) : [...current, path],
    );
  }

  async function handleScan() {
    await runAction('scan', async () => {
      setScanResult(await scanArtifacts(runOptions));
      setActivePane('scan');
    });
  }

  async function handleAnalyze() {
    await runAction('analyze', async () => {
      setAnalysisResult(await analyzeArtifacts(runOptions));
      setActivePane('analysis');
    });
  }

  async function handleBuildCleanupPlan() {
    await runAction('cleanup-plan', async () => {
      const result = await buildCleanupPlan(runOptions);
      setCleanupPlan(result);
      setSelectedCleanupPaths(result.candidates.map((candidate) => candidate.path));
      setActivePane('cleanup');
    });
  }

  async function handleAudit() {
    await runAction('audit', async () => {
      setAuditResult(await auditScripts(runOptions));
      setActivePane('audit');
    });
  }

  async function handleExecuteCleanup() {
    if (!cleanupPlan) {
      return;
    }

    const candidates = cleanupPlan.candidates.filter((candidate) =>
      selectedCleanupPaths.includes(candidate.path),
    );

    await runAction('cleanup-execute', async () => {
      const result = await executeCleanup(candidates, deleteMode);

      setCleanupResult(result);
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
      setActivePane('cleanup');
    });
  }

  function handlePrimaryPaneAction() {
    if (activePane === 'scan') {
      void handleScan();
      return;
    }
    if (activePane === 'analysis') {
      void handleAnalyze();
      return;
    }
    if (activePane === 'cleanup') {
      void handleBuildCleanupPlan();
      return;
    }
    void handleAudit();
  }

  function handleCleanupSelection() {
    if (activePane === 'cleanup' && selectedItem?.path) {
      toggleCleanupPath(selectedItem.path);
    }
  }

  return {
    root,
    search,
    activePane,
    selectedItemId,
    selectedItem,
    selectedEcosystems,
    deleteMode,
    busyAction,
    error,
    selectedCleanupPaths,
    selectedCandidateCount,
    currentItems,
    activePaneConfig,
    paneConfigs,
    canRunActions,
    statusMetrics,
    totalsMetrics,
    footerStats,
    statusMessage,
    primaryInspectorActionLabel: primaryActionLabel(activePane, busyAction),
    ecosystems,
    deleteModes,
    setRoot,
    setSearch,
    setActivePane,
    setSelectedItemId,
    setDeleteMode,
    toggleEcosystem,
    handleScan,
    handleAnalyze,
    handleBuildCleanupPlan,
    handleAudit,
    handleExecuteCleanup,
    handlePrimaryPaneAction,
    handleCleanupSelection,
  };
}
