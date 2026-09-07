import { act, fireEvent, render, renderHook, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { scanExecutableIntegrity } from '../../../lib/tauri';
import type {
  ExecutableObservation,
  IntegrityScanResponse,
  SignatureReport,
} from '../../../types/workflow';
import { defaultIntegrityTools, useExecutableIntegrity } from '../hooks/useExecutableIntegrity';
import { ExecutableIntegrityView } from './ExecutableIntegrityView';

vi.mock('../../../lib/tauri', () => ({
  scanExecutableIntegrity: vi.fn(),
}));

const hash = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

function observation(tool: string, path: string, currentHash = hash): ExecutableObservation {
  return {
    requestedTool: tool,
    resolvedPath: path,
    canonicalPath: path,
    sizeBytes: 128,
    sha256: currentHash,
    observedAt: '2026-09-07T01:02:03Z',
  };
}

function signature(status: SignatureReport['status'], platform: SignatureReport['platform']): SignatureReport {
  return {
    status,
    platform,
    verificationMessage: status === 'unsupported' ? 'No verifier is implemented.' : undefined,
  };
}

const scanResponse: IntegrityScanResponse = {
  checks: [
    {
      requestedTool: 'git',
      status: 'newBaseline',
      observation: observation('git', '/usr/bin/git'),
      signature: signature('valid', 'macos'),
    },
    {
      requestedTool: 'node',
      status: 'unchanged',
      observation: observation('node', '/usr/local/bin/node'),
      signature: signature('unsupported', 'linux'),
    },
    {
      requestedTool: 'cargo',
      status: 'contentChanged',
      observation: observation('cargo', '/Users/dev tools/bin/cargo', 'b'.repeat(64)),
      previousObservation: observation('cargo', '/Users/dev tools/bin/cargo'),
      signature: signature('unsigned', 'macos'),
    },
    {
      requestedTool: 'rustc',
      status: 'resolvedPathChanged',
      observation: observation('rustc', '/opt/rust/bin/rustc'),
      previousObservation: observation('rustc', '/usr/bin/rustc'),
      signature: signature('invalid', 'macos'),
    },
    {
      requestedTool: 'java',
      status: 'missing',
      failure: { kind: 'notFound', message: 'no PATH candidate found for java' },
    },
    {
      requestedTool: 'gradle',
      status: 'inspectionFailed',
      failure: { kind: 'unreadable', message: 'permission denied' },
      signature: signature('inspectionFailed', 'macos'),
    },
  ],
};

describe('ExecutableIntegrityView', () => {
  afterEach(() => vi.clearAllMocks());

  it('runs an explicit scan and renders structured integrity and signature evidence', async () => {
    vi.mocked(scanExecutableIntegrity).mockResolvedValue(scanResponse);

    render(<TestExecutableIntegrityView />);
    expect(screen.getByRole('heading', { name: 'Ready to inspect' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Run integrity scan' })).toBeEnabled();

    fireEvent.click(screen.getByRole('button', { name: 'Run integrity scan' }));

    await waitFor(() => expect(screen.getByRole('heading', { name: 'git' })).toBeInTheDocument());
    expect(scanExecutableIntegrity).toHaveBeenCalledWith({ tools: defaultIntegrityTools });
    expect(screen.getByText('6 tools checked')).toBeInTheDocument();
    expect(screen.getByText('Content changed')).toBeInTheDocument();
    expect(screen.getByText('Resolved path changed')).toBeInTheDocument();
    expect(screen.getByText('Missing')).toBeInTheDocument();
    expect(screen.getAllByText('Inspection failed')).not.toHaveLength(0);
    expect(screen.getAllByText('/Users/dev tools/bin/cargo')).not.toHaveLength(0);
    expect(screen.getAllByText(hash)).not.toHaveLength(0);
    expect(screen.getByText(/not proof of malware or compromise/)).toBeInTheDocument();
    expect(screen.getAllByText('Unsupported')).not.toHaveLength(0);
    expect(screen.getAllByText('Unsigned')).not.toHaveLength(0);
    expect(screen.getAllByText('Invalid')).not.toHaveLength(0);
    expect(screen.getAllByText('Inspection failed')).toHaveLength(3);
    expect(screen.getByText('no PATH candidate found for java')).toBeInTheDocument();
  });

  it('passes custom names and paths as one structured tool selection', async () => {
    vi.mocked(scanExecutableIntegrity).mockResolvedValue({ checks: [] });

    render(<TestExecutableIntegrityView />);
    const input = screen.getByLabelText('Additional tool name or path');
    fireEvent.change(input, { target: { value: '/tmp/tools/my tool' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'git' }));
    fireEvent.click(screen.getByRole('button', { name: 'Run integrity scan' }));

    await waitFor(() => expect(scanExecutableIntegrity).toHaveBeenCalled());
    expect(scanExecutableIntegrity).toHaveBeenCalledWith({
      tools: [...defaultIntegrityTools.filter((tool) => tool !== 'git'), '/tmp/tools/my tool'],
    });
  });

  it('does not let an older scan overwrite a newer result', async () => {
    const pending = new Map<string, (response: IntegrityScanResponse) => void>();
    vi.mocked(scanExecutableIntegrity).mockImplementation(({ tools }) =>
      new Promise((resolve) => pending.set(tools[0], resolve)),
    );

    const hook = renderHook(() => useExecutableIntegrity());
    act(() => {
      void hook.result.current.scan(['first']);
      void hook.result.current.scan(['second']);
    });

    await act(async () => {
      pending.get('second')?.({
        checks: [{ requestedTool: 'second', status: 'unchanged' }],
      });
    });
    await waitFor(() => expect(hook.result.current.result?.checks[0].requestedTool).toBe('second'));

    await act(async () => {
      pending.get('first')?.({
        checks: [{ requestedTool: 'first', status: 'contentChanged' }],
      });
    });

    expect(hook.result.current.result?.checks[0].requestedTool).toBe('second');
  });
});

function TestExecutableIntegrityView() {
  const integrity = useExecutableIntegrity();
  return <ExecutableIntegrityView integrity={integrity} />;
}
