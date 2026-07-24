import { StorageSummary } from '../../../components/StorageSummary/StorageSummary';
import { formatBytes, formatCount, formatDate } from '../../../lib/format';
import type { SidebarEntry } from '../../../components/Sidebar/Sidebar';

type OverviewViewProps = {
  sidebarEntries: SidebarEntry[];
  lastScanAtMs: number | null;
  reclaimableBytes: number;
  statusMessage: string;
  error: string | null;
};

export function OverviewView(props: OverviewViewProps) {
  const languageCount = props.sidebarEntries
    .filter((entry) => entry.section === 'language')
    .reduce((total, entry) => total + entry.count, 0);

  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Workspace Overview</p>
        <h2 className="mt-1 text-2xl font-semibold text-white">System Management Dashboard</h2>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-300">
          Inspect reclaimable storage, review scan results by category, and run safe cleanup flows
          with confirmation before files are moved to Trash.
        </p>
      </section>

      <StorageSummary
        metrics={[
          {
            label: 'Reclaimable Storage',
            value: formatBytes(props.reclaimableBytes),
          },
          {
            label: 'Detected Artifacts',
            value: formatCount(languageCount),
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
