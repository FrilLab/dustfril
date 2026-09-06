import { CleanupDialog } from '../../components/CleanupDialog/CleanupDialog';
import { Sidebar } from '../../components/Sidebar/Sidebar';
import { categoryConfig } from '../../model/categories';
import { pathBreadcrumb } from '../../model/presentation';
import { AppHeader } from './components/AppHeader';
import { useAppState } from './hooks/useAppState';
import { HistoryView } from './views/HistoryView';
import { DependenciesView } from './views/DependenciesView';
import { ModulePlaceholderView } from './views/ModulePlaceholderView';
import { OverviewView } from './views/OverviewView';
import { WorkspaceView } from './views/WorkspaceView';

export function AppShell() {
  const app = useAppState();
  const activeConfig = categoryConfig(app.activeCategory);
  const showingCleanup = activeConfig?.ecosystem !== undefined;
  const showingWorkspace = app.activeCategory === 'workspace' || showingCleanup;
  const showingDependencies = app.activeCategory === 'workspace-dependencies';
  const showingActivity =
    app.activeCategory === 'history' || app.activeCategory === 'workspace-activity';

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
              storageSummary={app.storageSummary}
              artifacts={app.analysisResult?.artifacts ?? []}
              candidates={app.cleanupPlan?.candidates ?? []}
              reclaimableBytes={app.cleanupPlan?.reclaimableSizeBytes ?? 0}
              historyEntries={app.historyEntries}
              error={app.error}
              onInspectArtifact={app.openWorkspaceArtifact}
              onOpenHistory={() => app.setActiveCategory('history')}
            />
          ) : null}

          {showingWorkspace ? (
            <WorkspaceView
              ecosystem={activeConfig?.ecosystem}
              artifacts={app.filteredArtifacts}
              candidates={app.cleanupPlan?.candidates ?? []}
              operationStatus={app.workspaceOperation.status}
              selectedItemId={app.selectedItemId}
              selectedPaths={app.selectedCleanupPaths}
              canReviewCleanup={app.canReviewCleanup}
              deleteMode={app.deleteMode}
              deleteModes={app.deleteModes}
              cleanupAgeDays={app.cleanupAgeDays}
              busy={app.busyAction !== null}
              analysisReady={app.analysisResult !== null}
              error={app.error}
              onSelectItem={app.setSelectedItemId}
              onCloseInspector={() => app.setSelectedItemId(null)}
              onTogglePath={app.toggleCleanupPath}
              onDeleteModeChange={app.setDeleteMode}
              onCleanupAgeChange={app.handleCleanupAgeChange}
              onRequestCleanup={app.handleRequestCleanup}
            />
          ) : null}

          {showingActivity ? (
            <HistoryView
              entries={app.historyEntries}
              busy={app.busyAction !== null}
              error={app.error}
              onClearHistory={app.handleClearHistory}
            />
          ) : null}

          {showingDependencies ? (
            <DependenciesView
              root={app.root}
              result={app.dependencyResult}
              operationStatus={app.dependencyOperation.status}
              busy={app.busyAction !== null}
              error={app.error}
              onLoad={app.handleLoadDependencyInventory}
              onCompare={app.handleCompareDependencyBaseline}
              onAccept={app.handleAcceptDependencyBaseline}
            />
          ) : null}

          {app.activeCategory !== 'overview' &&
          !showingWorkspace &&
          !showingActivity &&
          !showingDependencies &&
          activeConfig ? (
            <ModulePlaceholderView
              config={activeConfig}
              onReturnToOverview={() => app.setActiveCategory('overview')}
            />
          ) : null}
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
      </footer>

      <CleanupDialog
        open={app.confirmDialogOpen}
        itemCount={app.selectedCleanupItems.length}
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
