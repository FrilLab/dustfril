import { CleanupDialog } from '../../components/CleanupDialog/CleanupDialog';
import { Sidebar } from '../../components/Sidebar/Sidebar';
import { formatBytes } from '../../lib/format';
import { pathBreadcrumb } from '../../model/presentation';
import { AppHeader } from './components/AppHeader';
import { useAppState } from './hooks/useAppState';
import { HistoryView } from './views/HistoryView';
import { OverviewView } from './views/OverviewView';
import { WorkspaceView } from './views/WorkspaceView';

export function AppShell() {
  const app = useAppState();
  const historyCount = app.sidebarEntries.find((entry) => entry.key === 'history')?.count ?? 0;

  return (
    <main className="app-shell">
      <AppHeader
        root={app.root}
        search={app.search}
        busy={app.busyAction !== null}
        canAnalyze={app.canAnalyze}
        onChooseWorkspace={app.handleChooseWorkspace}
        onSearchChange={app.setSearch}
        onAnalyzeWorkspace={app.handleAnalyzeWorkspace}
      />

      <div className="app-content">
        <Sidebar
          entries={app.sidebarEntries}
          activeCategory={app.activeCategory}
          onCategoryChange={app.setActiveCategory}
        />

        <section className="main-pane">
          {app.activeCategory === 'overview' ? (
            <OverviewView
              root={app.root}
              analysisReady={app.analysisResult !== null}
              artifactCount={app.summary.artifactCount}
              reclaimableBytes={app.summary.reclaimableBytes}
              lastAnalysisAtMs={app.lastAnalysisAtMs}
              historyCount={historyCount}
              discoveredEcosystems={app.discoveredEcosystems}
              statusMessage={app.statusMessage}
              error={app.error}
            />
          ) : null}

          {app.activeCategory === 'workspace' ? (
            <WorkspaceView
              root={app.root}
              artifacts={app.filteredArtifacts}
              candidates={app.cleanupPlan?.candidates ?? []}
              reclaimableBytes={app.cleanupPlan?.reclaimableSizeBytes ?? 0}
              selectedItemId={app.selectedItemId}
              selectedPaths={app.selectedCleanupPaths}
              selectedBytes={app.selectedCandidateBytes}
              deleteMode={app.deleteMode}
              deleteModes={app.deleteModes}
              lastAnalysisAtMs={app.lastAnalysisAtMs}
              busy={app.busyAction !== null}
              analysisReady={app.analysisResult !== null}
              statusMessage={app.statusMessage}
              error={app.error}
              discoveredEcosystems={app.discoveredEcosystems}
              onSelectItem={app.setSelectedItemId}
              onTogglePath={app.toggleCleanupPath}
              onDeleteModeChange={app.setDeleteMode}
            />
          ) : null}

          {app.activeCategory === 'history' ? <HistoryView entries={app.historyEntries} /> : null}
        </section>
      </div>

      <footer className="app-statusbar">
        <div className="breadcrumb" title={app.root}>
          {app.root ? (
            pathBreadcrumb(app.root).map((segment, index) => (
              <span key={`${segment}-${index}`}>
                {index ? <span className="breadcrumb-separator">›</span> : null}
                {segment}
              </span>
            ))
          ) : (
            <span>No workspace selected</span>
          )}
        </div>
        <div className="statusbar-selection">
          <span>
            Selected {app.selectedCleanupPaths.length} · {formatBytes(app.selectedCandidateBytes)}
          </span>
          <button
            type="button"
            className="review-button"
            onClick={app.handleRequestCleanup}
            disabled={!app.canReviewCleanup}
          >
            Review Cleanup
          </button>
        </div>
      </footer>

      <CleanupDialog
        open={app.confirmDialogOpen}
        itemCount={app.selectedCleanupPaths.length}
        totalBytes={app.selectedCandidateBytes}
        deleteMode={app.deleteMode}
        samplePaths={app.confirmSamplePaths}
        busy={app.busyAction === 'cleanup-execute'}
        onCancel={() => app.setConfirmDialogOpen(false)}
        onConfirm={app.handleConfirmCleanup}
      />
    </main>
  );
}
