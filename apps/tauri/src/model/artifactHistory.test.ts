import { describe, expect, it } from 'vitest';
import { latestScanForWorkspace, scanExecutionLabel } from './artifactHistory';
import type { ActivityRecord } from '../types/workflow';

function scan(id: string, timestampMs: number, path: string): ActivityRecord {
  return {
    id,
    timestampMs,
    kind: 'Scan',
    result: { success: true, details: { path } },
  };
}

describe('artifact history model', () => {
  it('selects the newest scan for the selected workspace only', () => {
    const entries = [
      scan('other', 30, '/other'),
      scan('old', 10, '/workspace/'),
      scan('new', 20, '/workspace'),
    ];

    expect(latestScanForWorkspace(entries, '/workspace')?.id).toBe('new');
  });

  it('distinguishes completed scans with bounded access warnings', () => {
    const entry = scan('warning', 10, '/workspace');
    entry.result.details.accessSummary = {
      root: '/workspace',
      directoriesVisited: 1,
      filesInspected: 0,
      metadataFilesInspected: 0,
      artifactCandidates: 0,
      symlinksSkipped: 0,
      failures: 2,
      failureSamples: [],
    };

    expect(scanExecutionLabel(entry)).toBe('Completed with warnings');
    entry.result.success = false;
    expect(scanExecutionLabel(entry)).toBe('Failed');
  });
});
