import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { DeveloperCacheView } from './DeveloperCacheView';
import { discoverDeveloperCaches, executeDeveloperCacheCleanup } from '../../../lib/tauri';

vi.mock('../../../lib/tauri', () => ({
  discoverDeveloperCaches: vi.fn(),
  executeDeveloperCacheCleanup: vi.fn(),
}));

const registryCache = {
  kind: 'cargoRegistry' as const,
  tool: 'Cargo',
  name: 'Registry cache',
  path: '/fake/cargo-home/registry',
  scope: 'global' as const,
  sizeBytes: 4096,
  evidence: 'Resolved from the absolute CARGO_HOME environment variable.',
  supportState: 'supported' as const,
  cleanupImpact: 'May require downloads and can reduce offline availability.',
  measurementFailures: 0,
  failureSamples: [],
};

describe('DeveloperCacheView', () => {
  it('shows provider evidence and requires individual selection and confirmation', async () => {
    vi.mocked(discoverDeveloperCaches).mockResolvedValue({ caches: [registryCache], warnings: [] });
    vi.mocked(executeDeveloperCacheCleanup).mockResolvedValue({
      deletedPaths: ['/fake/cargo-home/registry'],
      failedPaths: [],
      freedSizeBytes: 4096,
    });
    const onHistoryRefresh = vi.fn();

    render(<DeveloperCacheView busy={false} onHistoryRefresh={onHistoryRefresh} />);

    const checkbox = await screen.findByRole('checkbox', { name: 'Select Cargo Registry cache' });
    expect(checkbox).not.toBeChecked();
    expect(screen.getAllByText(/global cache/i).length).toBeGreaterThan(0);
    expect(screen.getByText(registryCache.evidence)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Review cleanup' })).toBeDisabled();

    fireEvent.click(checkbox);
    fireEvent.click(screen.getByRole('button', { name: 'Review cleanup' }));

    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveTextContent('/fake/cargo-home/registry');
    expect(dialog).toHaveTextContent('Move to Trash?');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Move to Trash' }));

    await waitFor(() => expect(executeDeveloperCacheCleanup).toHaveBeenCalledWith(
      [{ kind: 'cargoRegistry', path: '/fake/cargo-home/registry' }],
      'Trash',
    ));
    await waitFor(() => expect(onHistoryRefresh).toHaveBeenCalledOnce());
    expect(await screen.findByText(/Moved 1 cache root/)).toBeInTheDocument();
  });
});
