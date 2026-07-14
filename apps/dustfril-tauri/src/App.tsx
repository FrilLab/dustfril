import { useEffect, useMemo, useRef, useState } from 'react';
import {
  analyzeArtifacts,
  auditScripts,
  buildCleanupPlan,
  defaultRoot,
  executeCleanup,
  scanArtifacts,
} from './lib/tauri';
import { formatAge, formatBytes, formatCount, formatDate } from './lib/format';
import './styles/style.css';
import type {
  AnalysisResponse,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  Ecosystem,
  LifecycleScript,
  Recommendation,
  RiskLevel,
  RunOptions,
  ScanResponse,
} from './types/workflow';
import { deleteModes, ecosystems } from './types/workflow';

type BrowserPane = 'scan' | 'analysis' | 'cleanup' | 'audit';

type BrowserItem = {
  id: string;
  title: string;
  subtitle: string;
  meta: string;
  badge: string;
  accent: string;
  kind: 'folder' | 'document' | 'warning' | 'safe';
  detailLines: string[];
  path?: string;
};

type PaneConfig = {
  key: BrowserPane;
  title: string;
  description: string;
  count: number;
  accent: string;
};

function App() {
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

  useEffect(() => {
    if (!root || initializedRef.current) {
      return;
    }

    initializedRef.current = true;

    runAction('bootstrap', async () => {
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

  const runOptions = useMemo<RunOptions>(
    () => ({
      root,
      ecosystems: selectedEcosystems,
    }),
    [root, selectedEcosystems],
  );

  const summary = useMemo(() => {
    const result = analysisResult;

    if (!result) {
      return {
        keepCount: 0,
        reviewCount: 0,
        safeCount: 0,
        reviewBytes: 0,
        safeBytes: 0,
      };
    }

    return result.artifacts.reduce(
      (accumulator, artifact) => {
        if (artifact.recommendation === 'Keep') {
          accumulator.keepCount += 1;
        }
        if (artifact.recommendation === 'NeedsReview') {
          accumulator.reviewCount += 1;
          accumulator.reviewBytes += artifact.sizeBytes;
        }
        if (artifact.recommendation === 'SafeToClean') {
          accumulator.safeCount += 1;
          accumulator.safeBytes += artifact.sizeBytes;
        }
        return accumulator;
      },
      {
        keepCount: 0,
        reviewCount: 0,
        safeCount: 0,
        reviewBytes: 0,
        safeBytes: 0,
      },
    );
  }, [analysisResult]);

  const selectedCandidateCount = selectedCleanupPaths.length;
  const selectedCandidateBytes = useMemo(() => {
    const plan = cleanupPlan;
    if (!plan) {
      return 0;
    }

    return plan.candidates
      .filter((candidate) => selectedCleanupPaths.includes(candidate.path))
      .reduce((total, candidate) => total + candidate.sizeBytes, 0);
  }, [cleanupPlan, selectedCleanupPaths]);

  const paneConfigs = useMemo<PaneConfig[]>(
    () => [
      {
        key: 'analysis',
        title: 'Artifact Library',
        description: 'Age, size, and recommendation applied to every artifact.',
        count: analysisResult?.artifacts.length ?? 0,
        accent: 'text-slate-100',
      },
      {
        key: 'cleanup',
        title: 'Cleanup Queue',
        description: 'Candidates staged for deletion execution.',
        count: cleanupPlan?.candidates.length ?? 0,
        accent: 'text-cyan-200',
      },
      {
        key: 'scan',
        title: 'Scan Index',
        description: 'Raw artifact paths discovered from the workspace scan.',
        count: scanResult?.artifacts.length ?? 0,
        accent: 'text-amber-100',
      },
      {
        key: 'audit',
        title: 'Script Audit',
        description: 'Lifecycle scripts with risk grading.',
        count: auditResult.length,
        accent: 'text-rose-100',
      },
    ],
    [analysisResult, auditResult.length, cleanupPlan, scanResult],
  );

  const analysisItems = useMemo<BrowserItem[]>(
    () =>
      (analysisResult?.artifacts ?? []).map((artifact) => ({
        id: `analysis:${artifact.path}`,
        title: leafName(artifact.path),
        subtitle: artifact.path,
        meta: `${formatBytes(artifact.sizeBytes)} · ${formatAge(artifact.ageDays)}`,
        badge: artifact.recommendation,
        accent: recommendationAccent(artifact.recommendation),
        kind: artifact.recommendation === 'SafeToClean' ? 'safe' : artifact.recommendation === 'NeedsReview' ? 'warning' : 'folder',
        path: artifact.path,
        detailLines: [
          `Ecosystem: ${artifact.ecosystem}`,
          `Recommendation: ${artifact.recommendation}`,
          `Size: ${formatBytes(artifact.sizeBytes)}`,
          `Modified: ${formatDate(artifact.lastModifiedMs)}`,
          `Age: ${formatAge(artifact.ageDays)}`,
        ],
      })),
    [analysisResult],
  );

  const cleanupItems = useMemo<BrowserItem[]>(
    () =>
      (cleanupPlan?.candidates ?? []).map((candidate) => {
        const selected = selectedCleanupPaths.includes(candidate.path);
        return {
          id: `cleanup:${candidate.path}`,
          title: leafName(candidate.path),
          subtitle: candidate.path,
          meta: `${formatBytes(candidate.sizeBytes)} · ${formatAge(candidate.ageDays)}`,
          badge: selected ? 'Selected' : 'Queued',
          accent: selected
            ? 'border-cyan-300/40 bg-cyan-400/14 text-cyan-50'
            : 'border-white/10 bg-white/6 text-slate-200',
          kind: selected ? 'safe' : 'folder',
          path: candidate.path,
          detailLines: [
            `Ecosystem: ${candidate.ecosystem}`,
            `Stage: ${selected ? 'Included in execution' : 'Not selected'}`,
            `Size: ${formatBytes(candidate.sizeBytes)}`,
            `Age: ${formatAge(candidate.ageDays)}`,
            `Delete mode: ${deleteMode}`,
          ],
        };
      }),
    [cleanupPlan, deleteMode, selectedCleanupPaths],
  );

  const scanItems = useMemo<BrowserItem[]>(
    () =>
      (scanResult?.artifacts ?? []).map((artifact) => ({
        id: `scan:${artifact.path}`,
        title: leafName(artifact.path),
        subtitle: artifact.path,
        meta: artifact.ecosystem,
        badge: 'Scanned',
        accent: 'border-amber-300/30 bg-amber-400/10 text-amber-50',
        kind: 'folder',
        path: artifact.path,
        detailLines: [`Ecosystem: ${artifact.ecosystem}`, `Path: ${artifact.path}`],
      })),
    [scanResult],
  );

  const auditItems = useMemo<BrowserItem[]>(
    () =>
      auditResult.map((script) => ({
        id: `audit:${script.package}:${script.scriptType}:${script.command}`,
        title: `${script.package} · ${script.scriptType}`,
        subtitle: script.command,
        meta: `Risk ${script.riskLevel}`,
        badge: script.riskLevel,
        accent: riskAccent(script.riskLevel),
        kind: script.riskLevel === 'High' ? 'warning' : 'document',
        detailLines: [
          `Package: ${script.package}`,
          `Script: ${script.scriptType}`,
          `Risk: ${script.riskLevel}`,
          `Command: ${script.command}`,
        ],
      })),
    [auditResult],
  );

  const currentItems = useMemo(() => {
    const itemsByPane: Record<BrowserPane, BrowserItem[]> = {
      analysis: analysisItems,
      cleanup: cleanupItems,
      scan: scanItems,
      audit: auditItems,
    };

    const normalizedQuery = search.trim().toLowerCase();
    const baseItems = itemsByPane[activePane];

    if (!normalizedQuery) {
      return baseItems;
    }

    return baseItems.filter((item) =>
      [item.title, item.subtitle, item.badge, item.meta].some((value) =>
        value.toLowerCase().includes(normalizedQuery),
      ),
    );
  }, [activePane, analysisItems, auditItems, cleanupItems, scanItems, search]);

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
    const plan = cleanupPlan;

    if (!plan) {
      return;
    }

    const candidates = plan.candidates.filter((candidate) =>
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

  function handleSecondaryPaneAction() {
    if (activePane === 'cleanup' && selectedItem?.path) {
      toggleCleanupPath(selectedItem.path);
    }
  }

  const statusMessage = error
    ? error
    : cleanupResult
      ? `Last cleanup freed ${formatBytes(cleanupResult.freedSizeBytes)} across ${cleanupResult.deletedPaths.length} paths.`
      : 'Workspace loaded. Use the left sidebar to rescan, analyze, queue cleanup, or audit scripts.';

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#3f3f46_0%,#232326_30%,#161618_62%,#0d0d0f_100%)] px-3 py-3 text-slate-100 md:px-5 md:py-5">
      <div className="mx-auto flex min-h-[calc(100vh-1.5rem)] w-full max-w-[1600px] flex-col overflow-hidden rounded-[30px] border border-white/10 bg-[#1c1c1e] shadow-[0_35px_120px_rgba(0,0,0,0.45)]">
        <header className="border-b border-white/8 bg-[linear-gradient(180deg,rgba(58,58,60,0.95),rgba(44,44,46,0.95))] px-4 py-3 md:px-5">
          <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <span className="h-3 w-3 rounded-full bg-[#ff5f57]" />
                <span className="h-3 w-3 rounded-full bg-[#febc2e]" />
                <span className="h-3 w-3 rounded-full bg-[#28c840]" />
              </div>
              <div>
                <p className="text-[11px] uppercase tracking-[0.28em] text-slate-400">
                  DustFril Desktop
                </p>
                <h1 className="text-lg font-semibold text-white">Workspace Browser</h1>
              </div>
            </div>

            <div className="grid gap-3 xl:grid-cols-[minmax(320px,1fr)_280px]">
              <label className="flex items-center gap-3 rounded-2xl border border-white/8 bg-black/20 px-4 py-3">
                <FolderIcon />
                <input
                  value={root}
                  onChange={(event) => setRoot(event.currentTarget.value)}
                  className="w-full bg-transparent text-sm text-white outline-none placeholder:text-slate-500"
                  placeholder="/path/to/workspace"
                />
              </label>
              <label className="flex items-center gap-3 rounded-2xl border border-white/8 bg-black/20 px-4 py-3">
                <SearchIcon />
                <input
                  value={search}
                  onChange={(event) => setSearch(event.currentTarget.value)}
                  className="w-full bg-transparent text-sm text-white outline-none placeholder:text-slate-500"
                  placeholder="Search current pane"
                />
              </label>
            </div>
          </div>
        </header>

        <section className="grid flex-1 overflow-hidden xl:grid-cols-[280px_minmax(0,1fr)]">
          <aside className="border-r border-white/8 bg-[linear-gradient(180deg,#242426,#1d1d20)] px-4 py-4">
            <div className="space-y-5">
              <div>
                <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
                  Views
                </p>
                <div className="space-y-1.5">
                  {paneConfigs.map((pane) => {
                    const active = pane.key === activePane;

                    return (
                      <button
                        key={pane.key}
                        type="button"
                        onClick={() => setActivePane(pane.key)}
                        className={`flex w-full items-center justify-between rounded-2xl px-3 py-3 text-left transition ${
                          active ? 'bg-[#3a3a3c] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]' : 'text-slate-300 hover:bg-white/6'
                        }`}
                      >
                        <div className="min-w-0">
                          <p className={`truncate text-sm font-medium ${pane.accent}`}>{pane.title}</p>
                          <p className="mt-1 truncate text-xs text-slate-500">{pane.description}</p>
                        </div>
                        <span className="ml-3 rounded-full bg-black/20 px-2.5 py-1 text-xs text-slate-300">
                          {formatCount(pane.count)}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>

              <div>
                <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
                  Ecosystems
                </p>
                <div className="flex flex-wrap gap-2">
                  {ecosystems.map((ecosystem) => {
                    const active = selectedEcosystems.includes(ecosystem);

                    return (
                      <button
                        key={ecosystem}
                        type="button"
                        onClick={() => toggleEcosystem(ecosystem)}
                        className={`rounded-full border px-3 py-1.5 text-xs transition ${
                          active
                            ? 'border-sky-300/35 bg-sky-400/12 text-sky-50'
                            : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/8'
                        }`}
                      >
                        {ecosystem}
                      </button>
                    );
                  })}
                </div>
              </div>

              <div>
                <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
                  Actions
                </p>
                <div className="grid gap-2">
                  <SidebarActionButton
                    label={busyAction === 'scan' ? 'Scanning...' : 'Run Scan'}
                    onClick={handleScan}
                    disabled={!canRunActions}
                  />
                  <SidebarActionButton
                    label={busyAction === 'analyze' ? 'Analyzing...' : 'Analyze'}
                    onClick={handleAnalyze}
                    disabled={!canRunActions}
                  />
                  <SidebarActionButton
                    label={busyAction === 'cleanup-plan' ? 'Preparing...' : 'Build Cleanup'}
                    onClick={handleBuildCleanupPlan}
                    disabled={!canRunActions}
                  />
                  <SidebarActionButton
                    label={busyAction === 'audit' ? 'Auditing...' : 'Audit Scripts'}
                    onClick={handleAudit}
                    disabled={!canRunActions}
                  />
                </div>
              </div>

              <div>
                <p className="mb-3 text-xs font-medium uppercase tracking-[0.24em] text-slate-500">
                  Delete Mode
                </p>
                <div className="grid grid-cols-2 gap-2">
                  {deleteModes.map((mode) => (
                    <button
                      key={mode}
                      type="button"
                      onClick={() => setDeleteMode(mode)}
                      className={`rounded-2xl px-3 py-2 text-sm transition ${
                        deleteMode === mode
                          ? 'bg-[#3a3a3c] text-white'
                          : 'bg-black/15 text-slate-300 hover:bg-white/8'
                      }`}
                    >
                      {mode}
                    </button>
                  ))}
                </div>
              </div>

              <div className="rounded-[22px] border border-white/8 bg-black/15 p-4">
                <p className="text-xs uppercase tracking-[0.2em] text-slate-500">Status</p>
                <div className="mt-3 grid gap-2 text-sm text-slate-300">
                  <StatusRow label="Keep" value={formatCount(summary.keepCount)} />
                  <StatusRow label="Review" value={`${formatCount(summary.reviewCount)} · ${formatBytes(summary.reviewBytes)}`} />
                  <StatusRow label="Queued" value={`${formatCount(selectedCandidateCount)} · ${formatBytes(selectedCandidateBytes)}`} />
                  <StatusRow label="Audit" value={formatCount(auditResult.length)} />
                </div>
              </div>
            </div>
          </aside>

          <div className="grid min-h-0 xl:grid-cols-[minmax(0,1.15fr)_360px]">
            <section className="min-h-0 border-r border-white/8">
              <div className="border-b border-white/8 bg-[#2b2b2e] px-4 py-3">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Current View</p>
                    <h2 className="mt-1 text-lg font-semibold text-white">{activePaneConfig.title}</h2>
                  </div>
                  <div className="rounded-full bg-black/20 px-3 py-1 text-xs text-slate-300">
                    {formatCount(currentItems.length)} visible
                  </div>
                </div>
              </div>

              <div className="grid min-h-0 md:grid-cols-[minmax(0,1fr)_220px]">
                <div className="min-h-0 overflow-y-auto">
                  {currentItems.length ? (
                    <div className="divide-y divide-white/6">
                      {currentItems.map((item) => {
                        const selected = item.id === selectedItemId;

                        return (
                          <button
                            key={item.id}
                            type="button"
                            onClick={() => setSelectedItemId(item.id)}
                            className={`flex w-full items-center gap-3 px-4 py-3 text-left transition ${
                              selected ? 'bg-[#4a4a4f]/70' : 'hover:bg-white/5'
                            }`}
                          >
                            <ItemIcon kind={item.kind} />
                            <div className="min-w-0 flex-1">
                              <p className="truncate text-sm font-medium text-white">{item.title}</p>
                              <p className="mt-0.5 truncate text-xs text-slate-400">{item.subtitle}</p>
                            </div>
                            <div className="hidden text-right md:block">
                              <p className="text-xs text-slate-300">{item.meta}</p>
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  ) : (
                    <EmptyState message="No items in this view. Run the action again or broaden the filter." />
                  )}
                </div>

                <div className="min-h-0 border-t border-white/8 bg-[#202023] md:border-l md:border-t-0">
                  <div className="border-b border-white/8 px-4 py-3">
                    <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Preview</p>
                  </div>
                  <div className="overflow-y-auto px-4 py-4">
                    {selectedItem ? (
                      <div>
                        <div className="flex items-start gap-3">
                          <ItemIcon kind={selectedItem.kind} large />
                          <div className="min-w-0">
                            <p className="break-all text-sm font-semibold text-white">
                              {selectedItem.title}
                            </p>
                            <p className="mt-1 break-all text-xs text-slate-400">
                              {selectedItem.subtitle}
                            </p>
                          </div>
                        </div>
                        <div className={`mt-4 inline-flex rounded-full border px-3 py-1 text-xs ${selectedItem.accent}`}>
                          {selectedItem.badge}
                        </div>
                        <div className="mt-4 space-y-2 text-sm text-slate-300">
                          {selectedItem.detailLines.map((line) => (
                            <p key={line}>{line}</p>
                          ))}
                        </div>
                      </div>
                    ) : (
                      <EmptyState message="Select an item to inspect details." compact />
                    )}
                  </div>
                </div>
              </div>
            </section>

            <aside className="min-h-0 overflow-y-auto bg-[linear-gradient(180deg,#222225,#1b1b1d)] px-4 py-4">
              <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Inspector</p>
                <h3 className="mt-2 text-xl font-semibold text-white">
                  {selectedItem ? selectedItem.title : activePaneConfig.title}
                </h3>
                <p className="mt-2 text-sm leading-6 text-slate-300">
                  {selectedItem
                    ? selectedItem.subtitle
                    : activePaneConfig.description}
                </p>

                <div className="mt-5 grid gap-2">
                  <button
                    type="button"
                    onClick={handlePrimaryPaneAction}
                    disabled={!canRunActions}
                    className="rounded-2xl bg-[#d1d1d6] px-4 py-3 text-sm font-medium text-slate-950 transition hover:bg-white disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {primaryActionLabel(activePane, busyAction)}
                  </button>
                  {activePane === 'cleanup' ? (
                    <button
                      type="button"
                      onClick={handleSecondaryPaneAction}
                      disabled={!selectedItem?.path}
                      className="rounded-2xl border border-white/10 bg-white/6 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      {selectedItem?.path && selectedCleanupPaths.includes(selectedItem.path)
                        ? 'Remove From Execution'
                        : 'Add To Execution'}
                    </button>
                  ) : null}
                  <button
                    type="button"
                    onClick={handleExecuteCleanup}
                    disabled={busyAction !== null || selectedCandidateCount === 0}
                    className="rounded-2xl border border-cyan-300/20 bg-cyan-400/10 px-4 py-3 text-sm font-medium text-cyan-50 transition hover:bg-cyan-400/18 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {busyAction === 'cleanup-execute' ? 'Cleaning...' : `Execute Cleanup (${selectedCandidateCount})`}
                  </button>
                </div>
              </section>

              <section className="mt-4 rounded-[24px] border border-white/8 bg-black/12 p-4">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Workspace Totals</p>
                <div className="mt-4 grid gap-3">
                  <MetricCard label="Analyzed Size" value={formatBytes(analysisResult?.totalSizeBytes ?? 0)} />
                  <MetricCard label="Safe To Clean" value={`${formatCount(summary.safeCount)} · ${formatBytes(summary.safeBytes)}`} />
                  <MetricCard label="Cleanup Pool" value={formatBytes(cleanupPlan?.reclaimableSizeBytes ?? 0)} />
                  <MetricCard label="Audit Findings" value={formatCount(auditResult.length)} />
                </div>
              </section>

              <section className="mt-4 rounded-[24px] border border-white/8 bg-black/12 p-4">
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Execution Log</p>
                <div className={`mt-3 rounded-2xl border p-4 text-sm leading-6 ${
                  error
                    ? 'border-rose-400/20 bg-rose-500/10 text-rose-100'
                    : 'border-emerald-400/15 bg-emerald-500/10 text-emerald-50'
                }`}>
                  {statusMessage}
                </div>
              </section>
            </aside>
          </div>
        </section>

        <footer className="border-t border-white/8 bg-[#2b2b2e] px-4 py-3">
          <div className="flex flex-col gap-2 text-xs text-slate-400 md:flex-row md:items-center md:justify-between">
            <p>{root || 'No workspace selected'}</p>
            <div className="flex flex-wrap gap-4">
              <span>{formatCount(summary.keepCount)} keep</span>
              <span>{formatCount(summary.reviewCount)} review</span>
              <span>{formatCount(cleanupPlan?.candidates.length ?? 0)} cleanup</span>
              <span>{formatCount(auditResult.length)} scripts</span>
            </div>
          </div>
        </footer>
      </div>
    </main>
  );
}

function SidebarActionButton(props: {
  label: string;
  onClick: () => void | Promise<void>;
  disabled: boolean;
}) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      disabled={props.disabled}
      className="rounded-2xl bg-black/15 px-3 py-2.5 text-left text-sm text-slate-200 transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {props.label}
    </button>
  );
}

function StatusRow(props: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-slate-400">{props.label}</span>
      <span className="text-right text-slate-200">{props.value}</span>
    </div>
  );
}

function MetricCard(props: { label: string; value: string }) {
  return (
    <article className="rounded-2xl border border-white/8 bg-white/4 p-3">
      <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{props.label}</p>
      <p className="mt-2 text-sm font-medium text-white">{props.value}</p>
    </article>
  );
}

function EmptyState(props: { message: string; compact?: boolean }) {
  return (
    <div
      className={`flex items-center justify-center px-6 text-center text-sm text-slate-500 ${
        props.compact ? 'min-h-[180px]' : 'min-h-[420px]'
      }`}
    >
      {props.message}
    </div>
  );
}

function ItemIcon(props: { kind: BrowserItem['kind']; large?: boolean }) {
  const size = props.large ? 'h-12 w-12' : 'h-8 w-8';

  if (props.kind === 'document') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-white/8 text-slate-200`}>
        <DocumentIcon />
      </div>
    );
  }

  if (props.kind === 'warning') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-amber-400/12 text-amber-100`}>
        <WarningIcon />
      </div>
    );
  }

  if (props.kind === 'safe') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-cyan-400/12 text-cyan-100`}>
        <SparkIcon />
      </div>
    );
  }

  return (
    <div className={`flex ${size} items-center justify-center rounded-xl bg-sky-400/12 text-sky-100`}>
      <FolderIcon />
    </div>
  );
}

function primaryActionLabel(activePane: BrowserPane, busyAction: string | null) {
  if (activePane === 'scan') {
    return busyAction === 'scan' ? 'Scanning...' : 'Refresh Scan Index';
  }
  if (activePane === 'analysis') {
    return busyAction === 'analyze' ? 'Analyzing...' : 'Refresh Analysis';
  }
  if (activePane === 'cleanup') {
    return busyAction === 'cleanup-plan' ? 'Preparing...' : 'Rebuild Cleanup Queue';
  }
  return busyAction === 'audit' ? 'Auditing...' : 'Refresh Script Audit';
}

function recommendationAccent(recommendation: Recommendation) {
  if (recommendation === 'Keep') {
    return 'border-emerald-400/25 bg-emerald-400/10 text-emerald-100';
  }
  if (recommendation === 'NeedsReview') {
    return 'border-amber-400/25 bg-amber-400/10 text-amber-100';
  }
  return 'border-cyan-400/25 bg-cyan-400/10 text-cyan-100';
}

function riskAccent(level: RiskLevel) {
  if (level === 'High') {
    return 'border-rose-400/30 bg-rose-400/10 text-rose-100';
  }
  if (level === 'Medium') {
    return 'border-amber-400/30 bg-amber-400/10 text-amber-100';
  }
  if (level === 'Low') {
    return 'border-emerald-400/30 bg-emerald-400/10 text-emerald-100';
  }
  return 'border-slate-400/20 bg-slate-400/10 text-slate-200';
}

function leafName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : path;
}

function FolderIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="M3.75 7.5a2.25 2.25 0 0 1 2.25-2.25h4.182a2.25 2.25 0 0 1 1.591.659l1.136 1.137a2.25 2.25 0 0 0 1.591.659H18a2.25 2.25 0 0 1 2.25 2.25v6A2.25 2.25 0 0 1 18 18.75H6A2.25 2.25 0 0 1 3.75 16.5v-9Z" />
    </svg>
  );
}

function DocumentIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="M7.5 3.75h6.879a2.25 2.25 0 0 1 1.591.659l2.621 2.621a2.25 2.25 0 0 1 .659 1.591V18A2.25 2.25 0 0 1 17 20.25H7A2.25 2.25 0 0 1 4.75 18V6A2.25 2.25 0 0 1 7 3.75Z" />
      <path d="M15 3.75V7.5a.75.75 0 0 0 .75.75h3.75" />
    </svg>
  );
}

function WarningIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="m12 4.5 8.25 14.25H3.75L12 4.5Z" />
      <path d="M12 9v4.5" />
      <path d="M12 16.5h.008v.008H12z" />
    </svg>
  );
}

function SparkIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="m12 3 1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9L12 3Z" />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5 text-slate-500" stroke="currentColor" strokeWidth="1.8">
      <path d="m21 21-4.35-4.35" />
      <circle cx="10.5" cy="10.5" r="6.75" />
    </svg>
  );
}

export default App;
