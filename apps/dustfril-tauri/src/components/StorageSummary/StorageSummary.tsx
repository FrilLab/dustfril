type SummaryMetric = {
  label: string;
  value: string;
};

type StorageSummaryProps = {
  title?: string;
  metrics: SummaryMetric[];
};

export function StorageSummary(props: StorageSummaryProps) {
  return (
    <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
      <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
        {props.title ?? 'Storage Summary'}
      </p>
      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {props.metrics.map((metric) => (
          <article key={metric.label} className="rounded-2xl border border-white/8 bg-white/4 p-3">
            <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{metric.label}</p>
            <p className="mt-2 text-sm font-medium text-white">{metric.value}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
