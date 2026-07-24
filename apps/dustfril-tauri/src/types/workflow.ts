export type Ecosystem = 'Rust' | 'Node' | 'Java';
export type Recommendation = 'Keep' | 'NeedsReview' | 'SafeToClean';
export type DeleteMode = 'Trash' | 'Permanent';
export type RiskLevel = 'Low' | 'Medium' | 'High' | 'None';

export type Artifact = {
  path: string;
  ecosystem: Ecosystem;
};

export type ScanResponse = {
  artifacts: Artifact[];
};

export type ArtifactAnalysis = {
  path: string;
  ecosystem: Ecosystem;
  sizeBytes: number;
  lastModifiedMs: number | null;
  ageDays: number | null;
  recommendation: Recommendation;
};

export type AnalysisResponse = {
  artifacts: ArtifactAnalysis[];
  totalSizeBytes: number;
};

export type CleanupCandidate = {
  path: string;
  ecosystem: Ecosystem;
  sizeBytes: number;
  ageDays: number | null;
};

export type CleanupPlanResponse = {
  candidates: CleanupCandidate[];
  reclaimableSizeBytes: number;
};

export type CleanupFailure = {
  path: string;
  reason: string;
};

export type CleanupResultResponse = {
  deletedPaths: string[];
  failedPaths: CleanupFailure[];
  freedSizeBytes: number;
};

export type LifecycleScript = {
  package: string;
  packageManager: string;
  scriptType: string;
  command: string;
  riskLevel: RiskLevel;
};

export type CleanupHistoryEntry = {
  executedAtMs: number;
  mode: DeleteMode;
  freedSizeBytes: number;
  deletedPaths: string[];
  failedPaths: string[];
};

export type RunOptions = {
  root: string;
  ecosystems: Ecosystem[];
};

export const ecosystems: Ecosystem[] = ['Rust', 'Node', 'Java'];
export const deleteModes: DeleteMode[] = ['Trash', 'Permanent'];
