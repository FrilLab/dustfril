import { riskTone } from '../lib/format';
import type { LifecycleScript } from '../types/workflow';

type AuditSectionProps = {
  auditScripts: LifecycleScript[];
};

export function AuditSection(props: AuditSectionProps) {
  return (
    <div className="rounded-[32px] border border-white/10 bg-white/6 p-6 shadow-[0_20px_80px_rgba(15,23,42,0.28)] backdrop-blur md:p-8">
      <p className="text-xs font-medium uppercase tracking-[0.24em] text-slate-400">Audit</p>
      <h3 className="mt-2 text-2xl font-semibold text-white">Lifecycle scripts</h3>
      <div className="mt-6 space-y-3">
        {props.auditScripts.length ? (
          props.auditScripts.map((script) => (
            <article
              key={`${script.package}-${script.scriptType}-${script.command}`}
              className="rounded-3xl border border-white/10 bg-slate-950/35 p-4"
            >
              <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200">
                  {script.package}
                </span>
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200">
                  {script.scriptType}
                </span>
                <span className={`rounded-full border px-3 py-1 text-xs ${riskTone(script.riskLevel)}`}>
                  {script.riskLevel}
                </span>
              </div>
              <p className="mt-3 break-all rounded-2xl bg-black/20 px-3 py-3 font-mono text-sm text-slate-100">
                {script.command}
              </p>
            </article>
          ))
        ) : (
          <div className="rounded-3xl border border-dashed border-white/10 bg-slate-950/20 p-6 text-sm text-slate-400">
            No lifecycle scripts loaded.
          </div>
        )}
      </div>
    </div>
  );
}
