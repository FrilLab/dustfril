import { CleanupDialog } from '../../components/CleanupDialog/CleanupDialog';
import { Dashboard } from '../../components/Dashboard/Dashboard';
import { Sidebar } from '../../components/Sidebar/Sidebar';
import { AppHeader } from './components/AppHeader';
import { useAppState } from './hooks/useAppState';
import { CategoryCleanupView } from './views/CategoryCleanupView';
import { HistoryView } from './views/HistoryView';
import { PlaceholderCategoryView } from './views/PlaceholderCategoryView';

export function AppShell() {
  const app = useAppState();

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#3f3f46_0%,#232326_30%,#161618_62%,#0d0d0f_100%)] px-3 py-3 text-slate-100 md:px-5 md:py-5">
      <div className="mx-auto flex min-h-[calc(100vh-1.5rem)] w-full max-w-[1600px] flex-col overflow-hidden rounded-[30px] border border-white/10 bg-[#1c1c1e] shadow-[0_35px_120px_rgba(0,0,0,0.45)]">
        <AppHeader
          root={app.root}
          search={app.search}
          onRootChange={app.setRoot}
          onSearchChange={app.setSearch}
        />

        <section className="grid flex-1 overflow-hidden xl:grid-cols-[280px_minmax(0,1fr)]">
          <Sidebar
            entries={app.sidebarEntries}
            activeCategory={app.activeCategory}
            onCategoryChange={app.setActiveCategory}
          />

          <div className="min-h-0 overflow-y-auto px-4 py-4">
            {app.activeCategory === 'overview' ? (
              <Dashboard
                sidebarEntries={app.sidebarEntries}
                lastScanAtMs={app.lastScanAtMs}
                reclaimableBytes={app.reclaimableBytes}
                artifactCount={app.artifactCount}
                supportedEcosystems={app.supportedEcosystems}
                statusMessage={app.statusMessage}
                error={app.error}
              />
            ) : null}

            {app.isLanguageCategory(app.activeCategory) ? (
              <CategoryCleanupView
                category={app.activeCategoryConfig}
                explorerWorkflow={app.explorerWorkflow}
                explorerItems={app.explorerItems}
                scanItems={app.scanItems}
                analysisItems={app.analysisItems}
                cleanupItems={app.cleanupItems}
                selectedItemId={app.selectedItemId}
                selectedCleanupPaths={app.selectedCleanupPaths}
                selectedCandidateBytes={app.selectedCandidateBytes}
                deleteMode={app.deleteMode}
                busyAction={app.busyAction}
                canRunActions={app.canRunActions}
                onWorkflowChange={app.setExplorerWorkflow}
                onSelectItem={app.setSelectedItemId}
                onToggleCleanupPath={app.toggleCleanupPath}
                onScanCategory={app.handleScanCategory}
                onAnalyzeCategory={app.handleAnalyzeCategory}
                onBuildCleanupPlan={app.handleBuildCleanupPlan}
                onRequestCleanup={app.handleRequestCleanup}
                onDeleteModeChange={app.setDeleteMode}
                deleteModes={app.deleteModes}
              />
            ) : null}

            {app.isFutureCategory(app.activeCategory) ? (
              <PlaceholderCategoryView category={app.activeCategoryConfig} />
            ) : null}

            {app.activeCategory === 'history' ? (
              <HistoryView entries={app.historyEntries} />
            ) : null}
          </div>
        </section>

        <footer className="border-t border-white/8 bg-[#2b2b2e] px-4 py-3">
          <div className="flex flex-col gap-2 text-xs text-slate-400 md:flex-row md:items-center md:justify-between">
            <p>{app.root || 'No workspace selected'}</p>
            <p>
              {app.summary.keepCount} keep · {app.summary.reviewCount} review ·{' '}
              {app.cleanupItems.length} cleanup · {app.historyEntries.length} history
            </p>
          </div>
        </footer>
      </div>

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
