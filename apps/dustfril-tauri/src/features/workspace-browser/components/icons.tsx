import type { BrowserItemKind } from '../model/types';

export function ItemIcon(props: { kind: BrowserItemKind; large?: boolean }) {
  const size = props.large ? 'h-12 w-12' : 'h-8 w-8';

  if (props.kind === 'document') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-white/8 text-slate-200`}>
        <DocumentIcon />
      </div>
    );
  }

  if (props.kind === 'warning') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-amber-400/12 text-amber-100`}>
        <WarningIcon />
      </div>
    );
  }

  if (props.kind === 'safe') {
    return (
      <div className={`flex ${size} items-center justify-center rounded-xl bg-cyan-400/12 text-cyan-100`}>
        <SparkIcon />
      </div>
    );
  }

  return (
    <div className={`flex ${size} items-center justify-center rounded-xl bg-sky-400/12 text-sky-100`}>
      <FolderIcon />
    </div>
  );
}

export function FolderIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="M3.75 7.5a2.25 2.25 0 0 1 2.25-2.25h4.182a2.25 2.25 0 0 1 1.591.659l1.136 1.137a2.25 2.25 0 0 0 1.591.659H18a2.25 2.25 0 0 1 2.25 2.25v6A2.25 2.25 0 0 1 18 18.75H6A2.25 2.25 0 0 1 3.75 16.5v-9Z" />
    </svg>
  );
}

function DocumentIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="M7.5 3.75h6.879a2.25 2.25 0 0 1 1.591.659l2.621 2.621a2.25 2.25 0 0 1 .659 1.591V18A2.25 2.25 0 0 1 17 20.25H7A2.25 2.25 0 0 1 4.75 18V6A2.25 2.25 0 0 1 7 3.75Z" />
      <path d="M15 3.75V7.5a.75.75 0 0 0 .75.75h3.75" />
    </svg>
  );
}

function WarningIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="m12 4.5 8.25 14.25H3.75L12 4.5Z" />
      <path d="M12 9v4.5" />
      <path d="M12 16.5h.008v.008H12z" />
    </svg>
  );
}

function SparkIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5" stroke="currentColor" strokeWidth="1.8">
      <path d="m12 3 1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9L12 3Z" />
    </svg>
  );
}

export function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5 text-slate-500" stroke="currentColor" strokeWidth="1.8">
      <path d="m21 21-4.35-4.35" />
      <circle cx="10.5" cy="10.5" r="6.75" />
    </svg>
  );
}
