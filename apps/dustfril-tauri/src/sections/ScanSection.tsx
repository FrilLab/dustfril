import type { ScanResponse } from '../types/workflow';

type ScanSectionProps = {
  scanResult: ScanResponse | null;
};

export function ScanSection(props: ScanSectionProps) {
  return (
    <div className="rounded-[32px] border border-white/10 bg-white/6 p-6 shadow-[0_20px_80px_rgba(15,23,42,0.28)] backdrop-blur md:p-8">
      <p className="text-xs font-medium uppercase tracking-[0.24em] text-slate-400">
        Scan Output
      </p>
      <h3 className="mt-2 text-2xl font-semibold text-white">Detected artifacts</h3>
      <div className="mt-6 space-y-3">
        {props.scanResult?.artifacts.length ? (
          props.scanResult.artifacts.map((artifact) => (
            <article key={artifact.path} className="rounded-3xl border border-white/10 bg-slate-950/35 p-4">
              <div className="flex items-center justify-between gap-3">
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200">
                  {artifact.ecosystem}
                </span>
              </div>
              <p className="mt-3 break-all text-sm font-medium text-white">{artifact.path}</p>
            </article>
          ))
        ) : (
          <div className="rounded-3xl border border-dashed border-white/10 bg-slate-950/20 p-6 text-sm text-slate-400">
            No scan results loaded.
          </div>
        )}
      </div>
    </div>
  );
}
