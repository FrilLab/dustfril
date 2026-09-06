import { describe, expect, it } from 'vitest';
import { categoryConfig, categoryConfigs, categorySections } from './categories';

describe('desktop module navigation', () => {
  it('declares the planned information architecture in one place', () => {
    expect(categorySections.map((section) => section.title)).toEqual([
      'Favorites',
      'Cleanup',
      'Workspace',
      'Security',
    ]);
    expect(categoryConfigs.map((config) => config.title)).toEqual([
      'Overview',
      'Workspace',
      'History',
      'Rust',
      'Node.js',
      'Java',
      'Cache',
      'Dependencies',
      'Artifact History',
      'Activity',
      'Supply Chain',
      'GitHub Actions',
      'Executable Integrity',
    ]);
  });

  it('marks only existing workflows as available', () => {
    expect(categoryConfig('cleanup-rust')?.availability).toBe('available');
    expect(categoryConfig('cleanup-node')?.availability).toBe('available');
    expect(categoryConfig('cleanup-java')?.availability).toBe('available');
    expect(categoryConfig('workspace-activity')?.availability).toBe('available');
    expect(categoryConfig('cleanup-cache')?.availability).toBe('planned');
    expect(categoryConfig('security-supply-chain')?.availability).toBe('planned');
    expect(categoryConfig('workspace-artifact-history')?.availability).toBe('planned');
  });
});
