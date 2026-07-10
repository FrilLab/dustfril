import { formatAge, formatBytes, formatDate, recommendationTone } from '../lib/format';
import type { AnalysisResponse } from '../types/workflow';

type AnalysisSectionProps = {
  analysisResult: AnalysisResponse | null;
};

export function AnalysisSection(props: AnalysisSectionProps) {
  return (
    <div className="rounded-[32px] border border-white/10 bg-white/6 p-6 shadow-[0_20px_80px_rgba(15,23,42,0.28)] backdrop-blur md:p-8">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.24em] text-slate-400">
            Analysis
          </p>
          <h3 className="mt-2 text-2xl font-semibold text-white">Artifact details</h3>
        </div>
        <div className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200">
          {props.analysisResult
            ? `${props.analysisResult.artifacts.length} entries`
            : 'Run analysis'}
        </div>
      </div>

      <div className="mt-6 space-y-3">
        {props.analysisResult?.artifacts.length ? (
          props.analysisResult.artifacts.map((artifact) => (
            <article key={artifact.path} className="rounded-3xl border border-white/10 bg-slate-950/35 p-4">
              <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-200">
                      {artifact.ecosystem}
                    </span>
                    <span
                      className={`rounded-full border px-3 py-1 text-xs ${recommendationTone(artifact.recommendation)}`}
                    >
                      {artifact.recommendation}
                    </span>
                  </div>
                  <p className="mt-3 break-all text-sm font-medium text-white">{artifact.path}</p>
                </div>
                <div className="text-left text-sm text-slate-300 md:text-right">
                  <p>{formatBytes(artifact.sizeBytes)}</p>
                  <p className="mt-1">{formatAge(artifact.ageDays)}</p>
                </div>
              </div>
              <div className="mt-4 grid gap-2 text-sm text-slate-400 md:grid-cols-2">
                <p>Last modified: {formatDate(artifact.lastModifiedMs)}</p>
                <p>Recommendation basis: age window from analyzer</p>
              </div>
            </article>
          ))
        ) : (
          <div className="rounded-3xl border border-dashed border-white/10 bg-slate-950/20 p-6 text-sm text-slate-400">
            No analysis loaded.
          </div>
        )}
      </div>
    </div>
  );
}
