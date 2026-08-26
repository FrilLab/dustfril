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

const commands = {
  defaultRoot: 'default_root',
  scan: 'scan',
  analyze: 'analyze',
  buildCleanupPlan: 'build_cleanup_plan',
  audit: 'audit',
  executeCleanup: 'execute_cleanup',
  loadCleanupHistory: 'load_cleanup_history',
} as const;

export function defaultRoot() {
  return invoke<string>(commands.defaultRoot);
}

export function scanArtifacts(options: RunOptions) {
  return invoke<ScanResponse>(commands.scan, { options });
}

export function analyzeArtifacts(options: RunOptions) {
  return invoke<AnalysisResponse>(commands.analyze, { options });
}

export function buildCleanupPlan(options: RunOptions) {
  return invoke<CleanupPlanResponse>(commands.buildCleanupPlan, { options });
}

export function auditScripts(options: RunOptions) {
  return invoke<LifecycleScript[]>(commands.audit, { options });
}

export function executeCleanup(candidates: CleanupCandidate[], mode: DeleteMode) {
  return invoke<CleanupResultResponse>(commands.executeCleanup, {
    request: { candidates, mode },
  });
}

export function loadCleanupHistory() {
  return invoke<CleanupHistoryEntry[]>(commands.loadCleanupHistory);
}
