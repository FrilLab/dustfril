import { useEffect, useMemo, useState } from 'react';
import HeaderLayout from './layout/Header';
import {
  analyzeArtifacts,
  auditScripts,
  buildCleanupPlan,
  defaultRoot,
  executeCleanup,
  scanArtifacts,
} from './lib/tauri';
import { AnalysisSection } from './sections/AnalysisSection';
import { AuditSection } from './sections/AuditSection';
import { CleanupSection } from './sections/CleanupSection';
import { ControlPanel } from './sections/ControlPanel';
import { LiveSummaryPanel } from './sections/LiveSummaryPanel';
import { ScanSection } from './sections/ScanSection';
import { StatsGrid } from './sections/StatsGrid';
import './styles/style.css';
import type {
  AnalysisResponse,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  Ecosystem,
  LifecycleScript,
  RunOptions,
  ScanResponse,
} from './types/workflow';
import { ecosystems } from './types/workflow';

function App() {
  const [root, setRoot] = useState('');
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
    });
  }

  async function handleAnalyze() {
    await runAction('analyze', async () => {
      setAnalysisResult(await analyzeArtifacts(runOptions));
    });
  }

  async function handleBuildCleanupPlan() {
    await runAction('cleanup-plan', async () => {
      const result = await buildCleanupPlan(runOptions);
      setCleanupPlan(result);
      setSelectedCleanupPaths(result.candidates.map((candidate) => candidate.path));
    });
  }

  async function handleAudit() {
    await runAction('audit', async () => {
      setAuditResult(await auditScripts(runOptions));
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
    });
  }

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(251,146,60,0.22),_transparent_28%),radial-gradient(circle_at_top_right,_rgba(56,189,248,0.16),_transparent_24%),linear-gradient(160deg,#020617_0%,#0f172a_55%,#111827_100%)] px-4 py-5 text-slate-50 md:px-8 md:py-8">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-5">
        <HeaderLayout />

        <section className="grid gap-5 xl:grid-cols-[1.35fr_0.65fr]">
          <ControlPanel
            root={root}
            selectedEcosystems={selectedEcosystems}
            deleteMode={deleteMode}
            busyAction={busyAction}
            onRootChange={setRoot}
            onToggleEcosystem={toggleEcosystem}
            onDeleteModeChange={setDeleteMode}
            onScan={handleScan}
            onAnalyze={handleAnalyze}
            onBuildCleanupPlan={handleBuildCleanupPlan}
            onAudit={handleAudit}
          />
          <LiveSummaryPanel
            scanCount={scanResult?.artifacts.length ?? 0}
            analyzedSizeBytes={analysisResult?.totalSizeBytes ?? 0}
            safeCount={summary.safeCount}
            safeBytes={summary.safeBytes}
            auditCount={auditResult.length}
            error={error}
          />
        </section>

        <StatsGrid
          keepCount={summary.keepCount}
          reviewCount={summary.reviewCount}
          reviewBytes={summary.reviewBytes}
          cleanupCount={cleanupPlan?.candidates.length ?? 0}
          cleanupBytes={cleanupPlan?.reclaimableSizeBytes ?? 0}
        />

        <section className="grid gap-5 xl:grid-cols-[1.2fr_0.8fr]">
          <AnalysisSection analysisResult={analysisResult} />
          <CleanupSection
            cleanupPlan={cleanupPlan}
            cleanupResult={cleanupResult}
            busyAction={busyAction}
            selectedCleanupPaths={selectedCleanupPaths}
            selectedCandidateCount={selectedCandidateCount}
            selectedCandidateBytes={selectedCandidateBytes}
            onToggleCleanupPath={toggleCleanupPath}
            onExecute={handleExecuteCleanup}
          />
        </section>

        <section className="grid gap-5 xl:grid-cols-[0.92fr_1.08fr]">
          <ScanSection scanResult={scanResult} />
          <AuditSection auditScripts={auditResult} />
        </section>
      </div>
    </main>
  );
}

export default App;
