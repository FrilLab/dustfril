import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CleanupDialog } from './CleanupDialog';

describe('CleanupDialog safety context', () => {
  const commonProps = {
    open: true,
    itemCount: 1,
    totalBytes: 1024,
    selectedItems: [],
    busy: false,
    onCancel: () => undefined,
    onConfirm: () => undefined,
  };

  it('warns at the permanent action boundary only', () => {
    const { rerender } = render(<CleanupDialog {...commonProps} deleteMode="Trash" />);

    expect(screen.getByText(/moved to the system Trash/)).toBeInTheDocument();
    expect(screen.queryByText(/cannot be undone/)).not.toBeInTheDocument();

    rerender(<CleanupDialog {...commonProps} deleteMode="Permanent" />);

    expect(screen.getByText(/cannot be undone/)).toBeInTheDocument();
  });
});
