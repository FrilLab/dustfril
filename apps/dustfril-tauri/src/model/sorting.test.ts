import { describe, expect, it } from 'vitest';
import type { ActivityRecord, ArtifactAnalysis } from '../types/workflow';
import { sortActivityRecords, sortArtifacts } from './sorting';

function artifact(
  path: string,
  project: string,
  sizeBytes: number,
  lastModifiedMs: number | null,
  recommendation: ArtifactAnalysis['recommendation'],
): ArtifactAnalysis {
  return {
    path: `/workspace/${project}/${path}`,
    ecosystem: path === 'node_modules' ? 'Node' : 'Rust',
    project: {
      root: `/workspace/${project}`,
      displayName: project,
      ecosystem: path === 'node_modules' ? 'Node' : 'Rust',
    },
    sizeBytes,
    lastModifiedMs,
    ageDays: null,
    recommendation,
  };
}

function activity(
  id: string,
  timestampMs: number,
  kind: ActivityRecord['kind'],
  details: ActivityRecord['result']['details'],
  success = true,
): ActivityRecord {
  return { id, timestampMs, kind, result: { success, details } };
}

describe('artifact sorting', () => {
  const artifacts = [
    artifact('target', 'Zeta', 672 * 1024 ** 2, 200, 'Keep'),
    artifact('node_modules', 'alpha', 11 * 1024 ** 3, 100, 'NeedsReview'),
    artifact('target', 'beta', 102 * 1024 ** 2, 300, 'SafeToClean'),
  ];

  it('sorts projects and artifacts case-insensitively in both directions', () => {
    expect(sortArtifacts(artifacts, { column: 'project', direction: 'asc' }).map((item) => item.project.displayName)).toEqual([
      'alpha',
      'beta',
      'Zeta',
    ]);
    expect(sortArtifacts(artifacts, { column: 'project', direction: 'desc' }).map((item) => item.project.displayName)).toEqual([
      'Zeta',
      'beta',
      'alpha',
    ]);

    expect(sortArtifacts(artifacts, { column: 'artifact', direction: 'asc' }).map((item) => item.path)).toEqual([
      '/workspace/alpha/node_modules',
      '/workspace/beta/target',
      '/workspace/Zeta/target',
    ]);
    expect(sortArtifacts(artifacts, { column: 'artifact', direction: 'desc' }).map((item) => item.path)).toEqual([
      '/workspace/Zeta/target',
      '/workspace/beta/target',
      '/workspace/alpha/node_modules',
    ]);
  });

  it('sorts size and modified values from their numeric data', () => {
    expect(sortArtifacts(artifacts, { column: 'size', direction: 'desc' }).map((item) => item.sizeBytes)).toEqual([
      11 * 1024 ** 3,
      672 * 1024 ** 2,
      102 * 1024 ** 2,
    ]);
    expect(sortArtifacts(artifacts, { column: 'modified', direction: 'asc' }).map((item) => item.lastModifiedMs)).toEqual([
      100,
      200,
      300,
    ]);
  });

  it('sorts status by cleanup priority and keeps equal values deterministic', () => {
    expect(sortArtifacts(artifacts, { column: 'status', direction: 'desc' }).map((item) => item.recommendation)).toEqual([
      'SafeToClean',
      'NeedsReview',
      'Keep',
    ]);
    expect(sortArtifacts(artifacts, { column: 'status', direction: 'asc' }).map((item) => item.recommendation)).toEqual([
      'Keep',
      'NeedsReview',
      'SafeToClean',
    ]);
  });
});

describe('activity sorting', () => {
  const entries = [
    activity('scan-large', 100, 'Scan', { path: '/workspace/one', artifacts: 2, size: 11 * 1024 ** 3 }),
    activity('cleanup-small', 300, 'Cleanup', {
      target: '/workspace/one',
      mode: 'trash',
      freed: 672 * 1024 ** 2,
      items: [{ path: '/workspace/one/target', status: 'succeeded' }],
    }),
    activity('scan-small', 200, 'Scan', { path: '/workspace/two', artifacts: 14, size: 102 * 1024 ** 2 }),
    activity('cleanup-failed', 400, 'Cleanup', {
      target: '/workspace/two',
      mode: 'permanent',
      freed: 672 * 1024 ** 2,
      items: [{ path: '/workspace/two/target', status: 'failed' }],
    }, false),
  ];

  it('sorts time, action, and result using semantic fields', () => {
    expect(sortActivityRecords(entries, { column: 'time', direction: 'desc' }).map((entry) => entry.id)).toEqual([
      'cleanup-failed',
      'cleanup-small',
      'scan-small',
      'scan-large',
    ]);
    expect(sortActivityRecords(entries, { column: 'action', direction: 'asc' }).map((entry) => entry.kind)).toEqual([
      'Cleanup',
      'Cleanup',
      'Scan',
      'Scan',
    ]);
    expect(sortActivityRecords(entries, { column: 'result', direction: 'desc' }).map((entry) => entry.id)).toEqual([
      'scan-large',
      'cleanup-small',
      'cleanup-failed',
      'scan-small',
    ]);
  });

  it('sorts status by operational severity', () => {
    expect(sortActivityRecords(entries, { column: 'status', direction: 'desc' }).map((entry) => entry.id)).toEqual([
      'cleanup-failed',
      'scan-small',
      'cleanup-small',
      'scan-large',
    ]);
  });
});
