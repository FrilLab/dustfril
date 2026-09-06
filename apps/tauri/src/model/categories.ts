import type { Ecosystem } from '../types/workflow';

export type SidebarCategory =
  | 'overview'
  | 'workspace'
  | 'history'
  | 'cleanup-rust'
  | 'cleanup-node'
  | 'cleanup-java'
  | 'cleanup-cache'
  | 'workspace-dependencies'
  | 'workspace-artifact-history'
  | 'workspace-activity'
  | 'security-supply-chain'
  | 'security-github-actions'
  | 'security-executable-integrity';

export type CategorySection = 'favorites' | 'cleanup' | 'workspace' | 'security';

export type CategoryAvailability = 'available' | 'planned';

export type CategoryConfig = {
  key: SidebarCategory;
  title: string;
  description: string;
  section: CategorySection;
  availability?: CategoryAvailability;
  ecosystem?: Ecosystem;
};

export const categoryConfigs: CategoryConfig[] = [
  {
    key: 'overview',
    title: 'Overview',
    description: 'Workspace summary',
    section: 'favorites',
    availability: 'available',
  },
  {
    key: 'workspace',
    title: 'Workspace',
    description: 'All analyzed project artifacts and cleanup controls',
    section: 'favorites',
    availability: 'available',
  },
  {
    key: 'history',
    title: 'History',
    description: 'Shortcut to workspace activity',
    section: 'favorites',
    availability: 'available',
  },
  {
    key: 'cleanup-rust',
    title: 'Rust',
    description: 'Analyze and clean Rust artifacts',
    section: 'cleanup',
    availability: 'available',
    ecosystem: 'Rust',
  },
  {
    key: 'cleanup-node',
    title: 'Node.js',
    description: 'Analyze and clean Node.js artifacts',
    section: 'cleanup',
    availability: 'available',
    ecosystem: 'Node',
  },
  {
    key: 'cleanup-java',
    title: 'Java',
    description: 'Analyze and clean Java artifacts',
    section: 'cleanup',
    availability: 'available',
    ecosystem: 'Java',
  },
  {
    key: 'cleanup-cache',
    title: 'Cache',
    description: 'Future cache cleanup surface',
    section: 'cleanup',
    availability: 'planned',
  },
  {
    key: 'workspace-dependencies',
    title: 'Dependencies',
    description: 'Manifest and lockfile inventory with explicit baseline comparison',
    section: 'workspace',
    availability: 'available',
  },
  {
    key: 'workspace-artifact-history',
    title: 'Artifact History',
    description: 'Future generated-artifact history and growth',
    section: 'workspace',
    availability: 'planned',
  },
  {
    key: 'workspace-activity',
    title: 'Activity',
    description: 'Previous scans and cleanup operations',
    section: 'workspace',
    availability: 'available',
  },
  {
    key: 'security-supply-chain',
    title: 'Supply Chain',
    description: 'Future dependency and lifecycle security analysis',
    section: 'security',
    availability: 'planned',
  },
  {
    key: 'security-github-actions',
    title: 'GitHub Actions',
    description: 'Future workflow security analysis',
    section: 'security',
    availability: 'planned',
  },
  {
    key: 'security-executable-integrity',
    title: 'Executable Integrity',
    description: 'Future executable integrity evidence',
    section: 'security',
    availability: 'planned',
  },
];

export const categorySections: Array<{ key: CategorySection; title: string }> = [
  { key: 'favorites', title: 'Favorites' },
  { key: 'cleanup', title: 'Cleanup' },
  { key: 'workspace', title: 'Workspace' },
  { key: 'security', title: 'Security' },
];

export function categoryConfig(category: SidebarCategory) {
  return categoryConfigs.find((config) => config.key === category);
}
