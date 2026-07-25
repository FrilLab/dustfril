import { StorageSummary } from '../StorageSummary/StorageSummary';
import { formatBytes, formatCount, formatDate } from '../../lib/format';
import type { SidebarEntry } from '../Sidebar/Sidebar';
import type { Ecosystem } from '../../types/workflow';

type DashboardProps = {
  sidebarEntries: SidebarEntry[];
  lastScanAtMs: number | null;
  reclaimableBytes: number;
  artifactCount: number;
  supportedEcosystems: Ecosystem[];
  statusMessage: string;
  error: string | null;
};

export function Dashboard(props: DashboardProps) {
  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">DustFril</p>
        <h2 className="mt-1 text-2xl font-semibold text-white">Desktop Dashboard</h2>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-300">
          Discover artifacts, analyze disk usage, review cleanup candidates, and inspect history
          from a Finder-like desktop interface powered by DustFril Core.
        </p>
      </section>

      <StorageSummary
        metrics={[
          {
            label: 'Reclaimable',
            value: formatBytes(props.reclaimableBytes),
          },
          {
            label: 'Artifacts',
            value: formatCount(props.artifactCount),
          },
          {
            label: 'Last Scan',
            value: props.lastScanAtMs ? formatDate(props.lastScanAtMs) : 'Not scanned yet',
          },
          {
            label: 'History Entries',
            value: formatCount(
              props.sidebarEntries.find((entry) => entry.key === 'history')?.count ?? 0,
            ),
          },
        ]}
      />

      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Supported Ecosystems</p>
        <div className="mt-3 flex flex-wrap gap-2">
          {props.supportedEcosystems.map((ecosystem) => (
            <span
              key={ecosystem}
              className="rounded-full border border-white/10 bg-white/6 px-3 py-1 text-xs text-slate-200"
            >
              {ecosystem}
            </span>
          ))}
        </div>
      </section>

      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Cleanup Summary</p>
        <div
          className={`mt-3 rounded-2xl border p-4 text-sm leading-6 ${
            props.error
              ? 'border-rose-400/20 bg-rose-500/10 text-rose-100'
              : 'border-emerald-400/15 bg-emerald-500/10 text-emerald-50'
          }`}
        >
          {props.statusMessage}
        </div>
      </section>
    </div>
  );
}
