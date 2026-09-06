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
  project: ProjectIdentity;
};

export type ProjectIdentity = {
  root: string;
  displayName: string;
  ecosystem: Ecosystem;
};

export type ScanResponse = {
  artifacts: Artifact[];
  historyWarning?: string;
  artifactSnapshot?: ArtifactSnapshotResult;
  artifactSnapshotWarning?: string;
};

export type ArtifactSnapshotStatus = 'baselineCreated' | 'compared';
export type ArtifactChangeKind =
  | 'new'
  | 'removed'
  | 'sizeIncreased'
  | 'sizeDecreased'
  | 'unchanged';

export type ArtifactSizeChange = {
  path: string;
  ecosystem: Ecosystem;
  kind: ArtifactChangeKind;
  previousSizeBytes: number | null;
  currentSizeBytes: number | null;
  deltaBytes: number;
};

export type ArtifactSnapshot = {
  workspaceId: string;
  timestamp: string;
  artifacts: Array<{
    path: string;
    ecosystem: Ecosystem;
    sizeBytes: number;
    lastModifiedMs: number | null;
    ageDays: number | null;
  }>;
};

export type ArtifactSnapshotResult = {
  status: ArtifactSnapshotStatus;
  snapshot: ArtifactSnapshot;
  previousSnapshot: ArtifactSnapshot | null;
  changes: ArtifactSizeChange[];
};

export type ArtifactAnalysis = {
  path: string;
  ecosystem: Ecosystem;
  project: ProjectIdentity;
  sizeBytes: number;
  lastModifiedMs: number | null;
  ageDays: number | null;
  recommendation: Recommendation;
};

export type AnalysisResponse = {
  artifacts: ArtifactAnalysis[];
  totalSizeBytes: number;
  historyWarning?: string;
};

export type WorkspaceAnalysisResponse = {
  analysis: AnalysisResponse;
  cleanupPlan: CleanupPlanResponse;
  storageSummary: StorageSummary;
  artifactSnapshot?: ArtifactSnapshotResult;
  artifactSnapshotWarning?: string;
};

export type VolumeStorage = {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
};

export type StorageSummary =
  | {
      status: 'available';
      totalBytes: number;
      usedBytes: number;
      availableBytes: number;
      detectedDevelopmentBytes: number;
      detectedSharePercent: number | null;
      partial: boolean;
      warnings: string[];
      recommendedBytes: number;
      scopePath: string;
      categories: Ecosystem[];
    }
  | {
      status: 'unavailable';
      reason: string;
    };

export type CleanupCandidate = {
  path: string;
  ecosystem: Ecosystem;
  project: ProjectIdentity;
  sizeBytes: number;
  ageDays: number | null;
  recommendation: Recommendation;
  selectedByDefault: boolean;
};

export type ArtifactSelection = Pick<Artifact, 'path' | 'ecosystem'>;

export type CleanupPlanResponse = {
  analysisId: string;
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

export type IntegrityScanOptions = {
  tools: string[];
};

export type IntegrityStatus =
  | 'newBaseline'
  | 'unchanged'
  | 'contentChanged'
  | 'resolvedPathChanged'
  | 'missing'
  | 'inspectionFailed';

export type IntegrityFailureKind =
  | 'invalidToolName'
  | 'notFound'
  | 'nonRegularFile'
  | 'nonExecutable'
  | 'unreadable'
  | 'brokenSymlink'
  | 'symlinkLoop'
  | 'hashFailed';

export type IntegrityFailure = {
  kind: IntegrityFailureKind;
  message: string;
};

export type ExecutableObservation = {
  requestedTool: string;
  resolvedPath: string;
  canonicalPath: string;
  symlinkTarget?: string;
  sizeBytes: number;
  sha256: string;
  observedAt: string;
  versionMetadata?: string;
};

export type SignatureStatus = 'valid' | 'unsigned' | 'invalid' | 'unsupported' | 'inspectionFailed';
export type SignaturePlatform = 'macos' | 'windows' | 'linux' | 'other';
export type SignatureFailureKind =
  | 'platformUnsupported'
  | 'verifierUnavailable'
  | 'verifierFailed'
  | 'targetMissing'
  | 'targetUnreadable'
  | 'targetNonRegularFile'
  | 'targetChangedDuringVerification';

export type SignatureFailure = {
  kind: SignatureFailureKind;
  message: string;
};

export type SignatureReport = {
  platform: SignaturePlatform;
  status: SignatureStatus;
  signer?: string;
  teamIdentifier?: string;
  verificationMessage?: string;
  verificationCode?: number;
  failure?: SignatureFailure;
};

export type IntegrityCheck = {
  requestedTool: string;
  status: IntegrityStatus;
  observation?: ExecutableObservation;
  previousObservation?: ExecutableObservation;
  failure?: IntegrityFailure;
  signature?: SignatureReport;
};

export type IntegrityScanResponse = {
  checks: IntegrityCheck[];
};

export type ActivityKind = 'Scan' | 'Cleanup' | 'Security';

export type ActivityFailure = {
  path: string;
  reason?: string;
};

export type CleanupActivityItem = {
  path: string;
  status: 'succeeded' | 'failed';
  project?: string;
  projectRoot?: string;
  ecosystem?: Ecosystem;
  size?: number;
  reason?: string;
};

export type ScanAccessFailure = {
  path: string;
  reason: string;
};

export type ScanAccessSummary = {
  root: string;
  directoriesVisited: number;
  filesInspected: number;
  metadataFilesInspected: number;
  artifactCandidates: number;
  symlinksSkipped: number;
  failures: number;
  failureSamples: ScanAccessFailure[];
};

export type ActivityDetails = {
  path?: string;
  target?: string;
  artifacts?: number;
  size?: number;
  accessSummary?: ScanAccessSummary;
  mode?: 'trash' | 'permanent';
  deleted?: string[];
  failed?: ActivityFailure[];
  freed?: number;
  items?: CleanupActivityItem[];
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
  recordHistory?: boolean;
  cleanupAgeDays?: number;
  recordArtifactSnapshot?: boolean;
};

export const ecosystems: Ecosystem[] = ['Rust', 'Node', 'Java'];
export const deleteModes: DeleteMode[] = ['Trash', 'Permanent'];
export const cleanupAgeOptions = [7, 14, 30, 60, 90] as const;
export const defaultCleanupAgeDays = cleanupAgeOptions[2];
