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
              artifacts={app.filteredArtifacts}
              artifactCount={app.summary.artifactCount}
              candidates={app.cleanupPlan?.candidates ?? []}
              reclaimableBytes={app.cleanupPlan?.reclaimableSizeBytes ?? 0}
              selectedItemId={app.selectedItemId}
              selectedPaths={app.selectedCleanupPaths}
              deleteMode={app.deleteMode}
              deleteModes={app.deleteModes}
              cleanupAgeDays={app.cleanupAgeDays}
              lastAnalysisAtMs={app.lastAnalysisAtMs}
              busy={app.busyAction !== null}
              analysisReady={app.analysisResult !== null}
              statusMessage={app.statusMessage}
              error={app.error}
              onSelectItem={app.setSelectedItemId}
              onCloseInspector={() => app.setSelectedItemId(null)}
              onTogglePath={app.toggleCleanupPath}
              onDeleteModeChange={app.setDeleteMode}
              onCleanupAgeChange={app.handleCleanupAgeChange}
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
          <span>Selected size {formatBytes(app.selectedCandidateBytes)}</span>
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
        selectedItems={app.selectedCleanupItems}
        busy={app.busyAction === 'cleanup-execute'}
        onCancel={() => app.setConfirmDialogOpen(false)}
        onConfirm={app.handleConfirmCleanup}
      />
    </main>
  );
}
