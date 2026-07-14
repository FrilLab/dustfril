export function SidebarActionButton(props: {
  label: string;
  onClick: () => void | Promise<void>;
  disabled: boolean;
}) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      disabled={props.disabled}
      className="rounded-2xl bg-black/15 px-3 py-2.5 text-left text-sm text-slate-200 transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {props.label}
    </button>
  );
}

export function StatusRow(props: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-slate-400">{props.label}</span>
      <span className="text-right text-slate-200">{props.value}</span>
    </div>
  );
}

export function MetricCard(props: { label: string; value: string }) {
  return (
    <article className="rounded-2xl border border-white/8 bg-white/4 p-3">
      <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{props.label}</p>
      <p className="mt-2 text-sm font-medium text-white">{props.value}</p>
    </article>
  );
}

export function EmptyState(props: { message: string; compact?: boolean }) {
  return (
    <div
      className={`flex items-center justify-center px-6 text-center text-sm text-slate-500 ${
        props.compact ? 'min-h-[180px]' : 'min-h-[420px]'
      }`}
    >
      {props.message}
    </div>
  );
}
