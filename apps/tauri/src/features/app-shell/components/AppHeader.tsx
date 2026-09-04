import { FolderIcon, SearchIcon } from '../../../components/icons';
import { leafName } from '../../../model/presentation';

type AppHeaderProps = {
  root: string;
  search: string;
  busy: boolean;
  canAnalyze: boolean;
  onChooseWorkspace: () => void | Promise<void>;
  onSearchChange: (value: string) => void;
  onAnalyzeWorkspace: () => void | Promise<void>;
};

export function AppHeader(props: AppHeaderProps) {
  const workspaceName = props.root ? leafName(props.root) : 'Choose Workspace';

  return (
    <header className="app-toolbar">
      <div className="toolbar-brand" aria-label="DustFril">
        <div className="brand-mark" aria-hidden="true">
          D
        </div>
        <span>DustFril</span>
      </div>

      <button
        type="button"
        className="workspace-picker"
        onClick={() => void props.onChooseWorkspace()}
        disabled={props.busy}
        title={props.root || 'Choose a workspace folder'}
      >
        <FolderIcon />
        <span className="workspace-picker-name">{workspaceName}</span>
        <span className="workspace-picker-chevron" aria-hidden="true">
          ⌄
        </span>
      </button>

      <label className="toolbar-search">
        <SearchIcon />
        <input
          value={props.search}
          onChange={(event) => props.onSearchChange(event.currentTarget.value)}
          placeholder="Search workspace"
          aria-label="Search workspace results"
          spellCheck={false}
        />
        {props.search ? (
          <button
            type="button"
            className="search-clear"
            onClick={() => props.onSearchChange('')}
            aria-label="Clear search"
          >
            ×
          </button>
        ) : null}
      </label>

      <button
        type="button"
        className="toolbar-analyze"
        onClick={() => void props.onAnalyzeWorkspace()}
        disabled={!props.canAnalyze}
      >
        {props.busy ? 'Analyzing…' : 'Analyze Workspace'}
      </button>
    </header>
  );
}
