export type Ecosystem = 'Rust' | 'Node' | 'Java';
export type Recommendation = 'Keep' | 'NeedsReview' | 'SafeToClean';
export type DeleteMode = 'Trash' | 'Permanent';
export type RiskLevel = 'Low' | 'Medium' | 'High' | 'Critical' | 'None';
export type PackageManager = 'npm' | 'pnpm' | 'yarn' | 'bun' | 'unknown';
export type ScriptType =
  | 'preinstall'
  | 'install'
  | 'postinstall'
  | 'prepare'
  | 'prepublish'
  | 'prepublishOnly';

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
  packageManager: PackageManager;
  scriptType: ScriptType;
  command: string;
  riskLevel: RiskLevel;
};

export type ActivityKind = 'Scan' | 'Cleanup' | 'Security';

export type ActivityFailure = {
  path: string;
  reason?: string;
};

export type ActivityDetails = {
  path?: string;
  artifacts?: number;
  size?: number;
  mode?: 'trash' | 'permanent';
  deleted?: string[];
  failed?: ActivityFailure[];
  freed?: number;
  [key: string]: unknown;
};

export type ActivityResult = {
  success: boolean;
  details: ActivityDetails;
};

export type ActivityRecord = {
  id: string;
  timestampMs: number;
  kind: ActivityKind;
  result: ActivityResult;
};

export type RunOptions = {
  root: string;
  ecosystems: Ecosystem[];
};

export const ecosystems: Ecosystem[] = ['Rust', 'Node', 'Java'];
export const deleteModes: DeleteMode[] = ['Trash', 'Permanent'];
