import { invoke } from '@tauri-apps/api/core';
import type {
  AnalysisResponse,
  CleanupCandidate,
  CleanupHistoryEntry,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  LifecycleScript,
  RunOptions,
  ScanResponse,
} from '../types/workflow';

export function defaultRoot() {
  return invoke<string>('default_root');
}

export function scanArtifacts(options: RunOptions) {
  return invoke<ScanResponse>('scan', { options });
}

export function analyzeArtifacts(options: RunOptions) {
  return invoke<AnalysisResponse>('analyze', { options });
}

export function buildCleanupPlan(options: RunOptions) {
  return invoke<CleanupPlanResponse>('build_cleanup_plan', { options });
}

export function auditScripts(options: RunOptions) {
  return invoke<LifecycleScript[]>('audit', { options });
}

export function executeCleanup(candidates: CleanupCandidate[], mode: DeleteMode) {
  return invoke<CleanupResultResponse>('execute_cleanup', {
    request: { candidates, mode },
  });
}

export function loadCleanupHistory() {
  return invoke<CleanupHistoryEntry[]>('load_cleanup_history');
}
