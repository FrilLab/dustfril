import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AsyncStatePanel } from './AsyncStatePanel';

describe('AsyncStatePanel', () => {
  it('distinguishes partial results from their warnings', () => {
    render(
      <AsyncStatePanel
        status="partial"
        title="Analysis complete"
        description="The primary result is available."
        warnings={['One auxiliary record could not be saved.']}
      />,
    );

    expect(screen.getByText('Partial')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Analysis complete' })).toBeInTheDocument();
    expect(screen.getByText('One auxiliary record could not be saved.')).toBeInTheDocument();
  });

  it('renders planned surfaces as unsupported instead of functional results', () => {
    render(
      <AsyncStatePanel
        status="unsupported"
        title="Cache cleanup is planned"
        description="This module is not available yet."
        actionLabel="Return to Overview"
        onAction={vi.fn()}
      />,
    );

    expect(screen.getByText('Planned')).toBeInTheDocument();
    expect(screen.getByText('This module is not available yet.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Return to Overview' })).toBeInTheDocument();
  });
});
