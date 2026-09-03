import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AnalysisResponse,
  ArtifactSelection,
  ActivityRecord,
  CleanupPlanResponse,
  CleanupResultResponse,
  DeleteMode,
  LifecycleScript,
  RunOptions,
  ScanResponse,
  SecurityScanResponse,
  WorkspaceAnalysisResponse,
} from '../types/workflow';

const commands = {
  defaultRoot: 'default_root',
  scan: 'scan',
  analyze: 'analyze',
  buildCleanupPlan: 'build_cleanup_plan',
  analyzeWorkspace: 'analyze_workspace',
  audit: 'audit',
  securityScan: 'security_scan',
  executeCleanup: 'execute_cleanup',
  loadActivityHistory: 'load_activity_history',
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

export function analyzeWorkspace(options: RunOptions) {
  return invoke<WorkspaceAnalysisResponse>(commands.analyzeWorkspace, { options });
}

export async function chooseWorkspaceFolder(defaultPath?: string) {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath,
    title: 'Choose Workspace',
  });

  return typeof selected === 'string' ? selected : null;
}

export function auditScripts(options: RunOptions) {
  return invoke<LifecycleScript[]>(commands.audit, { options });
}

export function securityScan(options: RunOptions) {
  return invoke<SecurityScanResponse>(commands.securityScan, { options });
}

export function executeCleanup(
  root: string,
  ecosystems: RunOptions['ecosystems'],
  selectedArtifacts: ArtifactSelection[],
  mode: DeleteMode,
) {
  return invoke<CleanupResultResponse>(commands.executeCleanup, {
    request: { root, ecosystems, selectedArtifacts, mode },
  });
}

export function loadActivityHistory() {
  return invoke<ActivityRecord[]>(commands.loadActivityHistory);
}
