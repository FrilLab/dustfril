import { fireEvent, render, screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { DependenciesView } from './DependenciesView';
import type {
  DependencyDiff,
  DependencyInventoryResponse,
  DependencyReport,
} from '../../../types/workflow';

function metric(value: number | null, status: 'available' | 'unknown' | 'unsupported', reason: string | null = null) {
  return { value, status, reason };
}

const nodeReport: DependencyReport = {
  ecosystem: 'Node',
  status: 'complete',
  manifest: '/workspace/package.json',
  manifestFormat: 'package.json',
  lockfile: {
    path: '/workspace/package-lock.json',
    kind: 'PackageLockJson',
    format: 'package-lock.json',
    status: 'parsed',
    reason: null,
  },
  directDependencyCounts: {
    dependencies: 1,
    devDependencies: 1,
    optionalDependencies: 0,
    peerDependencies: 0,
  },
  directDependencyTotal: 2,
  resolvedDependencyCount: metric(3, 'available'),
  transitiveDependencyCount: metric(1, 'available'),
  duplicateVersions: [{ name: 'shared', versions: ['1.0.0', '2.0.0'] }],
  resolvedDependencies: [
    {
      ecosystem: 'Node',
      name: 'shared',
      version: '1.0.0',
      source: 'https://registry.npmjs.org/shared',
      scope: 'direct',
    },
    {
      ecosystem: 'Node',
      name: 'shared',
      version: '2.0.0',
      source: null,
      scope: 'transitive',
    },
  ],
  warnings: [],
};

const incompleteRustReport: DependencyReport = {
  ecosystem: 'Rust',
  status: 'missingLockfile',
  manifest: '/workspace/Cargo.toml',
  manifestFormat: 'Cargo.toml',
  lockfile: {
    path: '/workspace/Cargo.lock',
    kind: 'CargoLock',
    format: 'Cargo.lock',
    status: 'missing',
    reason: 'Cargo.lock is missing',
  },
  directDependencyCounts: {
    dependencies: 1,
    'dev-dependencies': 0,
    'build-dependencies': 0,
  },
  directDependencyTotal: 1,
  resolvedDependencyCount: metric(null, 'unknown', 'Cargo.lock is missing'),
  transitiveDependencyCount: metric(null, 'unknown', 'Cargo.lock is missing'),
  duplicateVersions: [],
  resolvedDependencies: [],
  warnings: ['Cargo.lock is missing'],
};

const unsupportedReport: DependencyReport = {
  ecosystem: 'Rust',
  status: 'unsupported',
  manifest: '/workspace/Cargo.toml',
  manifestFormat: 'Cargo.toml',
  lockfile: null,
  directDependencyCounts: {},
  directDependencyTotal: 0,
  resolvedDependencyCount: metric(null, 'unsupported', 'Cargo workspaces are not supported'),
  transitiveDependencyCount: metric(null, 'unsupported', 'Cargo workspaces are not supported'),
  duplicateVersions: [],
  resolvedDependencies: [],
  warnings: ['Cargo workspaces are not supported'],
};

const diff: DependencyDiff = {
  workspaceId: 'v1:/workspace',
  baselineStatus: 'compared',
  added: [{
    kind: 'added',
    previous: null,
    current: {
      ecosystem: 'Node',
      name: 'added-package',
      version: '1.0.0',
      source: null,
      scope: 'direct',
    },
  }],
  removed: [{
    kind: 'removed',
    previous: {
      ecosystem: 'Node',
      name: 'removed-package',
      version: '1.0.0',
      source: null,
      scope: 'transitive',
    },
    current: null,
  }],
  versionChanges: [{
    kind: 'versionChanged',
    previous: {
      ecosystem: 'Node',
      name: 'versioned-package',
      version: '1.0.0',
      source: null,
      scope: 'direct',
    },
    current: {
      ecosystem: 'Node',
      name: 'versioned-package',
      version: '2.0.0',
      source: null,
      scope: 'direct',
    },
  }],
  sourceChanges: [{
    kind: 'sourceChanged',
    previous: {
      ecosystem: 'Node',
      name: 'source-package',
      version: '1.0.0',
      source: 'registry-a',
      scope: 'transitive',
    },
    current: {
      ecosystem: 'Node',
      name: 'source-package',
      version: '1.0.0',
      source: 'registry-b',
      scope: 'transitive',
    },
  }],
  warnings: [],
};

const result: DependencyInventoryResponse = {
  inventoryFingerprint: 'fingerprint-1',
  workspacePath: '/workspace',
  reports: [nodeReport, incompleteRustReport],
  diff: null,
};

function renderView(overrides: Partial<ComponentProps<typeof DependenciesView>> = {}) {
  return render(
    <DependenciesView
      root="/workspace"
      result={result}
      operationStatus="success"
      busy={false}
      error={null}
      onLoad={vi.fn()}
      onCompare={vi.fn()}
      onAccept={vi.fn()}
      {...overrides}
    />,
  );
}

describe('DependenciesView', () => {
  it('renders category counts, duplicate versions, scopes, and unavailable metrics faithfully', () => {
    renderView();

    expect(screen.getByRole('heading', { name: 'Dependencies' })).toBeInTheDocument();
    expect(screen.getAllByText('dependencies').length).toBeGreaterThan(0);
    expect(screen.getByText('devDependencies')).toBeInTheDocument();
    expect(screen.getAllByText('Duplicate resolved versions').length).toBeGreaterThan(0);
    expect(screen.getAllByText('shared').length).toBeGreaterThan(0);
    expect(screen.getByText('1.0.0 · 2.0.0')).toBeInTheDocument();
    expect(screen.getAllByText('Direct').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Transitive').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Unknown').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Cargo.lock is missing').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Resolved packages').length).toBeGreaterThan(0);
  });

  it('renders all factual baseline change groups and keeps comparison context visible', () => {
    renderView({ result: { ...result, diff } });

    expect(screen.getByText('Baseline comparison')).toBeInTheDocument();
    expect(screen.getByText('v1:/workspace')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Added' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Removed' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Version changed' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Source changed' })).toBeInTheDocument();
    expect(screen.getByText('added-package')).toBeInTheDocument();
    expect(screen.getByText('removed-package')).toBeInTheDocument();
    expect(screen.getByText('1.0.0 → 2.0.0')).toBeInTheDocument();
    expect(screen.getByText(/1\.0\.0 · registry-a/)).toBeInTheDocument();
    expect(screen.getByText(/1\.0\.0 · registry-b/)).toBeInTheDocument();
    expect(screen.getByText(/Compare does not update the accepted baseline/)).toBeInTheDocument();
  });

  it('requires deliberate actions and does not compare or accept while rendering', () => {
    const onLoad = vi.fn();
    const onCompare = vi.fn();
    const onAccept = vi.fn();
    renderView({ onLoad, onCompare, onAccept });

    expect(onLoad).not.toHaveBeenCalled();
    expect(onCompare).not.toHaveBeenCalled();
    expect(onAccept).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Load Inventory' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare Baseline' }));
    fireEvent.click(screen.getByRole('button', { name: 'Accept Baseline' }));

    expect(onLoad).toHaveBeenCalledOnce();
    expect(onCompare).toHaveBeenCalledOnce();
    expect(onAccept).toHaveBeenCalledOnce();
  });

  it('does not turn malformed input into a clean empty inventory', () => {
    renderView({
      result: null,
      operationStatus: 'error',
      error: 'malformed package-lock.json',
    });

    expect(screen.getByRole('heading', { name: 'Dependency inventory failed' })).toBeInTheDocument();
    expect(screen.getAllByText('malformed package-lock.json').length).toBeGreaterThan(0);
    expect(screen.queryByText('No logical dependency changes detected.')).not.toBeInTheDocument();
    expect(screen.queryByText('0 dependencies')).not.toBeInTheDocument();
  });

  it('renders unsupported reports separately from missing lockfiles', () => {
    renderView({ result: { ...result, reports: [unsupportedReport] } });

    expect(screen.getByRole('heading', { name: 'Unsupported' })).toBeInTheDocument();
    expect(screen.getAllByText('Cargo workspaces are not supported').length).toBeGreaterThan(0);
    expect(screen.queryByText('Missing lockfile')).not.toBeInTheDocument();
  });

  it('keeps first-observation and unavailable baseline states explicit', () => {
    const firstObservation = renderView({
      result: {
        ...result,
        diff: {
          ...diff,
          baselineStatus: 'baselineCreated',
          added: [],
          removed: [],
          versionChanges: [],
          sourceChanges: [],
        },
      },
    });

    expect(screen.getByText(/first complete observation established the baseline/)).toBeInTheDocument();
    expect(screen.getByText('No logical dependency changes detected.')).toBeInTheDocument();
    firstObservation.unmount();

    renderView({
      result: {
        ...result,
        reports: [incompleteRustReport],
        diff: {
          ...diff,
          baselineStatus: 'unavailable',
          added: [],
          removed: [],
          versionChanges: [],
          sourceChanges: [],
        },
      },
    });

    expect(screen.getByText(/Comparison is unavailable/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Accept Baseline' })).toBeDisabled();
  });
});
