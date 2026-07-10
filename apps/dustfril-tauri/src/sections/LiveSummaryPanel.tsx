import { SummaryCard } from '../components/SummaryCard';
import { formatBytes, formatCount } from '../lib/format';

type LiveSummaryPanelProps = {
  scanCount: number;
  analyzedSizeBytes: number;
  safeCount: number;
  safeBytes: number;
  auditCount: number;
  error: string | null;
};

export function LiveSummaryPanel(props: LiveSummaryPanelProps) {
  return (
    <aside className="rounded-[32px] border border-white/10 bg-slate-950/45 p-6 shadow-[0_20px_80px_rgba(15,23,42,0.32)] backdrop-blur md:p-8">
      <p className="text-xs font-medium uppercase tracking-[0.24em] text-slate-400">Live Summary</p>
      <div className="mt-5 grid gap-3">
        <SummaryCard label="Scanned Artifacts" value={formatCount(props.scanCount)} />
        <SummaryCard label="Analyzed Size" value={formatBytes(props.analyzedSizeBytes)} />
        <SummaryCard
          label="Safe To Clean"
          value={formatCount(props.safeCount)}
          detail={formatBytes(props.safeBytes)}
        />
        <SummaryCard label="Audit Findings" value={formatCount(props.auditCount)} />
      </div>

      {props.error ? (
        <div className="mt-5 rounded-3xl border border-rose-400/25 bg-rose-500/10 p-4 text-sm text-rose-100">
          {props.error}
        </div>
      ) : (
        <div className="mt-5 rounded-3xl border border-emerald-400/20 bg-emerald-500/10 p-4 text-sm text-emerald-100">
          Cleanup stays explicit. Only `SafeToClean` candidates enter the plan, and delete mode
          remains visible before execution.
        </div>
      )}
    </aside>
  );
}
