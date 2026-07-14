import { FolderIcon, SearchIcon } from './icons';

type WorkspaceHeaderProps = {
  root: string;
  search: string;
  onRootChange: (value: string) => void;
  onSearchChange: (value: string) => void;
};

export function WorkspaceHeader(props: WorkspaceHeaderProps) {
  return (
    <header className="border-b border-white/8 bg-[linear-gradient(180deg,rgba(58,58,60,0.95),rgba(44,44,46,0.95))] px-4 py-3 md:px-5">
      <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="h-3 w-3 rounded-full bg-[#ff5f57]" />
            <span className="h-3 w-3 rounded-full bg-[#febc2e]" />
            <span className="h-3 w-3 rounded-full bg-[#28c840]" />
          </div>
          <div>
            <p className="text-[11px] uppercase tracking-[0.28em] text-slate-400">DustFril Desktop</p>
            <h1 className="text-lg font-semibold text-white">Workspace Browser</h1>
          </div>
        </div>

        <div className="grid gap-3 xl:grid-cols-[minmax(320px,1fr)_280px]">
          <label className="flex items-center gap-3 rounded-2xl border border-white/8 bg-black/20 px-4 py-3">
            <FolderIcon />
            <input
              value={props.root}
              onChange={(event) => props.onRootChange(event.currentTarget.value)}
              className="w-full bg-transparent text-sm text-white outline-none placeholder:text-slate-500"
              placeholder="/path/to/workspace"
            />
          </label>
          <label className="flex items-center gap-3 rounded-2xl border border-white/8 bg-black/20 px-4 py-3">
            <SearchIcon />
            <input
              value={props.search}
              onChange={(event) => props.onSearchChange(event.currentTarget.value)}
              className="w-full bg-transparent text-sm text-white outline-none placeholder:text-slate-500"
              placeholder="Search current pane"
            />
          </label>
        </div>
      </div>
    </header>
  );
}
