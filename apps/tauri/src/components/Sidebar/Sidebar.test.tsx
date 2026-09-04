import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Sidebar, type SidebarEntry } from './Sidebar';

const entries: SidebarEntry[] = [
  {
    key: 'overview',
    title: 'Overview',
    description: 'Summary',
    section: 'favorites',
    count: null,
  },
  {
    key: 'workspace',
    title: 'Workspace',
    description: 'Artifacts',
    section: 'favorites',
    count: 2,
  },
  {
    key: 'history',
    title: 'History',
    description: 'Activity',
    section: 'favorites',
    count: 3,
  },
];

describe('Sidebar information hierarchy', () => {
  it('does not render a permanent Trash guidance note', () => {
    render(
      <Sidebar
        entries={entries}
        activeCategory="workspace"
        onCategoryChange={vi.fn()}
      />,
    );

    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
    expect(screen.queryByText('Trash is the default cleanup mode.')).not.toBeInTheDocument();
  });
});
