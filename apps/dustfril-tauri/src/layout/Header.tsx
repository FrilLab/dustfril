const featureLabels = ['Scan', 'Analyze', 'Cleanup', 'Audit'];

export default function HeaderLayout() {
  return (
    <header className="flex flex-col gap-4 rounded-[28px] border border-white/10 bg-white/6 px-6 py-5 shadow-[0_20px_80px_rgba(15,23,42,0.35)] backdrop-blur md:flex-row md:items-center md:justify-between md:px-8">
      <div className="flex items-center gap-4">
        <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-[linear-gradient(135deg,#f97316,#fb7185)] text-lg font-semibold text-slate-950 shadow-[0_12px_30px_rgba(249,115,22,0.35)]">
          DF
        </div>
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.28em] text-orange-200/80">
            DustFril Desktop
          </p>
          <h1 className="text-xl font-semibold text-white">Core Workflow Console</h1>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {featureLabels.map((label) => (
          <span
            key={label}
            className="rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-200"
          >
            {label}
          </span>
        ))}
      </div>
    </header>
  );
}
