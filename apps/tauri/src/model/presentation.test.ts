import { describe, expect, it } from 'vitest';
import { selectedCandidateBytes } from './presentation';
import type { CleanupCandidate } from '../types/workflow';

function candidate(path: string, sizeBytes: number): CleanupCandidate {
  return {
    path,
    ecosystem: 'Node',
    project: { root: '/workspace', displayName: 'workspace', ecosystem: 'Node' },
    sizeBytes,
    ageDays: 30,
    recommendation: 'SafeToClean',
    selectedByDefault: true,
  };
}

describe('selectedCandidateBytes', () => {
  it('does not double-count a selected nested artifact', () => {
    expect(
      selectedCandidateBytes(
        [
          candidate('/workspace/node_modules', 100),
          candidate('/workspace/node_modules/package-a/node_modules', 25),
          candidate('/workspace/other/node_modules', 50),
        ],
        [
          '/workspace/node_modules',
          '/workspace/node_modules/package-a/node_modules',
          '/workspace/other/node_modules',
        ],
      ),
    ).toBe(150);
  });
});
