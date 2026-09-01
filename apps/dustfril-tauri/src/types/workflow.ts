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
  historyWarning?: string;
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
  historyWarning?: string;
};

export type LifecycleScript = {
  package: string;
  packageManager: PackageManager;
  scriptType: ScriptType;
  command: string;
  riskLevel: RiskLevel;
};

export type SecurityFinding = {
  path: string;
  rule: string;
  package: string | null;
  riskLevel: RiskLevel;
  evidence: string | null;
  reason: string;
};

export type SecurityWarning = {
  package: string;
  scriptType: string;
  command: string;
  riskLevel: RiskLevel;
};

export type LockfileCheck = {
  path: string;
  kind: 'PackageLockJson' | 'PnpmLockYaml' | 'BunLock' | 'CargoLock';
  status: 'Missing' | 'Modified' | 'Untracked' | 'Clean';
};

export type SecurityScanResponse = {
  findings: SecurityFinding[];
  lifecycleWarnings: SecurityWarning[];
  lockfiles: LockfileCheck[];
  manifests: string[];
  historyWarning?: string;
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
  ecosystems?: Ecosystem[];
  findingCount?: number;
  highestRisk?: RiskLevel;
  findings?: SecurityActivityFinding[];
  manifests?: number;
  lockfiles?: number;
  reason?: string;
  [key: string]: unknown;
};

export type SecurityActivityFinding = {
  rule: string;
  risk: RiskLevel;
  source: string;
  package?: string | null;
  reason: string;
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
