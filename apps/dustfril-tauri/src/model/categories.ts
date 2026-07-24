import type { Ecosystem } from '../types/workflow';

export type SidebarCategory =
  | 'overview'
  | 'rust'
  | 'node'
  | 'java'
  | 'system'
  | 'cache'
  | 'history';

export type CategorySection = 'primary' | 'language' | 'future';

export type CategoryConfig = {
  key: SidebarCategory;
  title: string;
  description: string;
  section: CategorySection;
  ecosystem?: Ecosystem;
  futureItems?: string[];
};

export const categoryConfigs: CategoryConfig[] = [
  {
    key: 'overview',
    title: 'Overview',
    description: 'Reclaimable storage and workspace summary',
    section: 'primary',
  },
  {
    key: 'rust',
    title: 'Rust',
    description: 'target/, Cargo registry cache, Cargo git cache',
    section: 'language',
    ecosystem: 'Rust',
  },
  {
    key: 'node',
    title: 'Node.js',
    description: 'node_modules/, npm cache, pnpm store, yarn cache, bun cache',
    section: 'language',
    ecosystem: 'Node',
  },
  {
    key: 'java',
    title: 'Java',
    description: 'build/, .gradle cache, Maven repository',
    section: 'language',
    ecosystem: 'Java',
  },
  {
    key: 'system',
    title: 'System',
    description: 'Temporary files, logs, application cache',
    section: 'future',
    futureItems: ['Temporary files', 'Logs', 'Application cache'],
  },
  {
    key: 'cache',
    title: 'Cache',
    description: 'Package manager and build caches',
    section: 'future',
    futureItems: ['Cargo registry', 'npm cache', 'pnpm store'],
  },
  {
    key: 'history',
    title: 'History',
    description: 'Previous cleanup operations',
    section: 'primary',
  },
];

export function ecosystemForCategory(category: SidebarCategory): Ecosystem | null {
  const config = categoryConfigs.find((entry) => entry.key === category);
  return config?.ecosystem ?? null;
}

export function isLanguageCategory(
  category: SidebarCategory,
): category is 'rust' | 'node' | 'java' {
  return category === 'rust' || category === 'node' || category === 'java';
}

export function isFutureCategory(category: SidebarCategory): category is 'system' | 'cache' {
  return category === 'system' || category === 'cache';
}
