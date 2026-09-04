export type SidebarCategory =
  | 'overview'
  | 'workspace'
  | 'history';

export type CategorySection = 'favorites';

export type CategoryConfig = {
  key: SidebarCategory;
  title: string;
  description: string;
  section: CategorySection;
};

export const categoryConfigs: CategoryConfig[] = [
  {
    key: 'overview',
    title: 'Overview',
    description: 'Workspace summary',
    section: 'favorites',
  },
  {
    key: 'workspace',
    title: 'Workspace',
    description: 'Analyze and clean project artifacts',
    section: 'favorites',
  },
  {
    key: 'history',
    title: 'History',
    description: 'Previous scans and cleanup operations',
    section: 'favorites',
  },
];
