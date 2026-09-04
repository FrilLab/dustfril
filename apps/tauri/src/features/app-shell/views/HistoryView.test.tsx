import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { HistoryView } from './HistoryView';
import type { ActivityRecord } from '../../../types/workflow';

const entry: ActivityRecord = {
  id: 'scan-1',
  timestampMs: Date.UTC(2026, 8, 3, 4, 5),
  kind: 'Scan',
  result: {
    success: true,
    details: { path: '/workspace/dustfril', artifacts: 1, size: 1024 },
  },
};

describe('HistoryView', () => {
  it('requires confirmation and keeps records when clearing is cancelled', () => {
    const onClearHistory = vi.fn().mockResolvedValue(undefined);

    render(
      <HistoryView
        entries={[entry]}
        busy={false}
        error={null}
        onClearHistory={onClearHistory}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Clear History' }));
    expect(screen.getByRole('heading', { name: 'Clear Activity History?' })).toBeInTheDocument();
    expect(screen.getByText(/It does not delete project files/)).toBeInTheDocument();

    fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Cancel' }));

    expect(onClearHistory).not.toHaveBeenCalled();
    expect(screen.getByText('dustfril')).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Clear Activity History?' })).not.toBeInTheDocument();
  });

  it('updates to the empty state immediately after a successful clear', async () => {
    function Harness() {
      const [entries, setEntries] = useState([entry]);

      return (
        <HistoryView
          entries={entries}
          busy={false}
          error={null}
          onClearHistory={async () => setEntries([])}
        />
      );
    }

    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: 'Clear History' }));
    fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Clear History' }));

    await waitFor(() => expect(screen.getByText('No activity yet')).toBeInTheDocument());
    expect(screen.getByText('Scans and cleanup operations will appear here.')).toBeInTheDocument();
    expect(screen.queryByText('dustfril')).not.toBeInTheDocument();
  });

  it('keeps records and shows a clear failure in the dialog', async () => {
    function Harness() {
      const [error, setError] = useState<string | null>(null);

      return (
        <HistoryView
          entries={[entry]}
          busy={false}
          error={error}
          onClearHistory={async () => {
            const message = 'History is unavailable.';
            setError(message);
            throw new Error(message);
          }}
        />
      );
    }

    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: 'Clear History' }));
    fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Clear History' }));

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('History is unavailable.'));
    expect(screen.getByText('dustfril')).toBeInTheDocument();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
