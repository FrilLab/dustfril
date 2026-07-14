import { formatBytes } from '../lib/format';

type StatsGridProps = {
  keepCount: number;
  reviewCount: number;
  reviewBytes: number;
  cleanupCount: number;
  cleanupBytes: number;
};

export function StatsGrid(props: StatsGridProps) {
  return (
    <section className="grid gap-5 lg:grid-cols-3">
      <article className="rounded-[28px] border border-white/10 bg-white/6 p-5 backdrop-blur">
        <p className="text-sm text-slate-400">Keep</p>
        <p className="mt-2 text-3xl font-semibold text-white">{props.keepCount}</p>
        <p className="mt-2 text-sm text-slate-300">Artifacts touched within the keep window.</p>
      </article>
      <article className="rounded-[28px] border border-white/10 bg-white/6 p-5 backdrop-blur">
        <p className="text-sm text-slate-400">Needs Review</p>
        <p className="mt-2 text-3xl font-semibold text-white">{props.reviewCount}</p>
        <p className="mt-2 text-sm text-slate-300">{formatBytes(props.reviewBytes)}</p>
      </article>
      <article className="rounded-[28px] border border-white/10 bg-white/6 p-5 backdrop-blur">
        <p className="text-sm text-slate-400">Cleanup Queue</p>
        <p className="mt-2 text-3xl font-semibold text-white">{props.cleanupCount}</p>
        <p className="mt-2 text-sm text-slate-300">{formatBytes(props.cleanupBytes)}</p>
      </article>
    </section>
  );
}
