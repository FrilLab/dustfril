import { useCallback, useEffect, useMemo, useState } from 'react';
import { formatBytes } from '../../../lib/format';
import {
  discoverDeveloperCaches,
  executeDeveloperCacheCleanup,
} from '../../../lib/tauri';
import type {
  DeleteMode,
  DeveloperCache,
  DeveloperCacheSelection,
  DeveloperCacheDiscovery,
  CleanupResultResponse,
} from '../../../types/workflow';

type DeveloperCacheViewProps = {
  busy: boolean;
  onHistoryRefresh: () => void | Promise<void>;
};

export function DeveloperCacheView(props: DeveloperCacheViewProps) {
  const [discovery, setDiscovery] = useState<DeveloperCacheDiscovery | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<DeleteMode>('Trash');
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [result, setResult] = useState<CleanupResultResponse | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setDiscovery(await discoverDeveloperCaches());
    } catch (invokeError) {
      setError(String(invokeError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const caches = discovery?.caches ?? [];
  const selectedCaches = useMemo(
    () => caches.filter((cache) => selectedIds.includes(cacheId(cache))),
    [caches, selectedIds],
  );
  const selectedBytes = selectedCaches.reduce((total, cache) => total + cache.sizeBytes, 0);
  const disabled = props.busy || executing;

  function toggleCache(cache: DeveloperCache) {
    const id = cacheId(cache);
    setSelectedIds((current) =>
      current.includes(id) ? current.filter((selected) => selected !== id) : [...current, id],
    );
  }

  async function executeCleanup() {
    if (selectedCaches.length === 0 || disabled) {
      return;
    }

    const selections: DeveloperCacheSelection[] = selectedCaches.map(({ kind, path }) => ({
      kind,
      path,
    }));
    setExecuting(true);
    setError(null);
    setResult(null);
    try {
      const cleanupResult = await executeDeveloperCacheCleanup(selections, mode);
      setResult(cleanupResult);
      setConfirmOpen(false);
      setSelectedIds([]);
      await Promise.all([refresh(), props.onHistoryRefresh()]);
    } catch (invokeError) {
      setError(String(invokeError));
    } finally {
      setExecuting(false);
    }
  }

  return (
    <section className="developer-cache-view" aria-labelledby="developer-cache-title">
      <header className="developer-cache-header">
        <div>
          <p className="eyebrow">Cleanup / Global caches</p>
          <h1 id="developer-cache-title">Developer caches</h1>
          <p>
            Review Cargo registry and Git caches resolved from <code>CARGO_HOME</code> or Cargo&apos;s
            platform default. Package-manager caches with unresolved configuration are not included.
          </p>
        </div>
        <button type="button" className="button-secondary" onClick={() => void refresh()} disabled={loading || disabled}>
          {loading ? 'Refreshing…' : 'Refresh'}
        </button>
      </header>

      {error ? <div className="developer-cache-notice developer-cache-error" role="alert">{error}</div> : null}
      {discovery?.warnings.map((warning) => (
        <div className="developer-cache-notice" role="status" key={warning}>{warning}</div>
      ))}
      {result ? <CleanupOutcome result={result} mode={mode} /> : null}

      {loading && !discovery ? (
        <div className="developer-cache-state" role="status">Resolving supported cache roots…</div>
      ) : null}

      {!loading && caches.length === 0 ? (
        <div className="developer-cache-state">
          <strong>No supported Cargo cache roots were found.</strong>
          <span>Missing cache directories are a normal empty state. Other package managers are not currently resolved.</span>
        </div>
      ) : null}

      {caches.length ? (
        <div className="developer-cache-list" aria-label="Supported developer caches">
          {caches.map((cache) => (
            <CacheCard
              key={cacheId(cache)}
              cache={cache}
              selected={selectedIds.includes(cacheId(cache))}
              disabled={disabled}
              onToggle={() => toggleCache(cache)}
            />
          ))}
        </div>
      ) : null}

      <div className="developer-cache-controls">
        <div className="cleanup-mode-control">
          <span className="control-label">MODE</span>
          <div className="mode-toggle" role="group" aria-label="Cache cleanup mode">
            {(['Trash', 'Permanent'] as const).map((deleteMode) => (
              <button
                key={deleteMode}
                type="button"
                className={mode === deleteMode ? 'mode-active' : ''}
                aria-pressed={mode === deleteMode}
                disabled={disabled}
                onClick={() => setMode(deleteMode)}
              >
                {deleteMode === 'Trash' ? 'Move to Trash' : 'Delete permanently'}
              </button>
            ))}
          </div>
        </div>
        <div className="cleanup-selection-summary">
          <span>{selectedCaches.length} selected · {formatBytes(selectedBytes)}</span>
          <button
            type="button"
            className="review-button"
            disabled={selectedCaches.length === 0 || disabled}
            onClick={() => setConfirmOpen(true)}
          >
            Review cleanup
          </button>
        </div>
      </div>

      {confirmOpen ? (
        <CacheCleanupConfirmation
          caches={selectedCaches}
          mode={mode}
          busy={executing}
          onCancel={() => setConfirmOpen(false)}
          onConfirm={() => void executeCleanup()}
        />
      ) : null}
    </section>
  );
}

function CacheCard({
  cache,
  selected,
  disabled,
  onToggle,
}: {
  cache: DeveloperCache;
  selected: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  return (
    <article className="developer-cache-card">
      <label className="developer-cache-select">
        <input
          type="checkbox"
          checked={selected}
          disabled={disabled}
          onChange={onToggle}
          aria-label={`Select ${cache.tool} ${cache.name}`}
        />
        <span>
          <strong>{cache.tool} · {cache.name}</strong>
          <small>{cache.scope} cache · Supported</small>
        </span>
        <strong className="developer-cache-size">{formatBytes(cache.sizeBytes)}</strong>
      </label>
      <dl>
        <div><dt>Path</dt><dd title={cache.path}>{cache.path}</dd></div>
        <div><dt>Resolved by</dt><dd>{cache.evidence}</dd></div>
        <div><dt>Impact</dt><dd>{cache.cleanupImpact}</dd></div>
      </dl>
      {cache.measurementFailures ? (
        <div className="developer-cache-notice" role="status">
          Could not read {cache.measurementFailures} entries while measuring this cache.
          {cache.failureSamples.length ? <span> Examples: {cache.failureSamples.join(', ')}</span> : null}
        </div>
      ) : null}
    </article>
  );
}

function CacheCleanupConfirmation({
  caches,
  mode,
  busy,
  onCancel,
  onConfirm,
}: {
  caches: DeveloperCache[];
  mode: DeleteMode;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const trash = mode === 'Trash';
  const totalBytes = caches.reduce((total, cache) => total + cache.sizeBytes, 0);
  return (
    <div className="dialog-backdrop">
      <div role="dialog" aria-modal="true" aria-labelledby="cache-cleanup-title" className="cleanup-dialog">
        <p className="eyebrow">Confirm Cache Cleanup</p>
        <h2 id="cache-cleanup-title">{trash ? 'Move to Trash?' : 'Delete permanently?'}</h2>
        <p className="dialog-copy">
          {trash
            ? 'The selected global caches will move to the system Trash. Cargo may download or rebuild their contents again.'
            : 'This permanently deletes the selected global caches. Cargo may download or rebuild their contents again, and offline use may be affected.'}
        </p>
        <div className="dialog-summary"><span>{caches.length} selected</span><strong>{formatBytes(totalBytes)}</strong></div>
        <ul className="developer-cache-confirm-list">
          {caches.map((cache) => (
            <li key={cacheId(cache)}>
              <strong>{cache.tool} · {cache.name}</strong>
              <code>{cache.path}</code>
              <small>{cache.cleanupImpact}</small>
            </li>
          ))}
        </ul>
        <div className="dialog-actions">
          <button type="button" onClick={onCancel} disabled={busy} className="button-secondary">Cancel</button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={busy}
            className={`button-confirm${trash ? '' : ' button-confirm-danger'}`}
          >
            {busy ? 'Cleaning…' : trash ? 'Move to Trash' : 'Delete permanently'}
          </button>
        </div>
      </div>
    </div>
  );
}

function CleanupOutcome({ result, mode }: { result: CleanupResultResponse; mode: DeleteMode }) {
  return (
    <div className={`developer-cache-outcome${result.failedPaths.length ? ' developer-cache-error' : ''}`} role="status">
      <strong>
        {result.deletedPaths.length
          ? `${mode === 'Trash' ? 'Moved' : 'Deleted'} ${result.deletedPaths.length} cache root${result.deletedPaths.length === 1 ? '' : 's'} · ${formatBytes(result.freedSizeBytes)} reclaimed`
          : 'No cache roots were cleaned'}
      </strong>
      {result.failedPaths.map((failure) => (
        <div key={`${failure.path}-${failure.reason}`}>{failure.path}: {failure.reason}</div>
      ))}
      {result.historyWarning ? <div>{result.historyWarning}</div> : null}
    </div>
  );
}

function cacheId(cache: Pick<DeveloperCache, 'kind' | 'path'>) {
  return `${cache.kind}:${cache.path}`;
}
