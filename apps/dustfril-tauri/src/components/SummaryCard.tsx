type SummaryCardProps = {
  label: string;
  value: string;
  detail?: string;
};

export function SummaryCard(props: SummaryCardProps) {
  return (
    <article className="rounded-3xl border border-white/10 bg-white/5 p-4">
      <p className="text-sm text-slate-400">{props.label}</p>
      <p className="mt-2 text-2xl font-semibold text-white">{props.value}</p>
      {props.detail ? <p className="mt-1 text-sm text-slate-300">{props.detail}</p> : null}
    </article>
  );
}
