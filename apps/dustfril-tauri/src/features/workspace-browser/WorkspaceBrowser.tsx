import { WorkspaceHeader } from './components/WorkspaceHeader';
import { WorkspaceSidebar } from './components/WorkspaceSidebar';
import { BrowserPane } from './components/BrowserPane';
import { WorkspaceInspector } from './components/WorkspaceInspector';
import { useWorkspaceBrowser } from './hooks/useWorkspaceBrowser';

export function WorkspaceBrowser() {
  const browser = useWorkspaceBrowser();

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#3f3f46_0%,#232326_30%,#161618_62%,#0d0d0f_100%)] px-3 py-3 text-slate-100 md:px-5 md:py-5">
      <div className="mx-auto flex min-h-[calc(100vh-1.5rem)] w-full max-w-[1600px] flex-col overflow-hidden rounded-[30px] border border-white/10 bg-[#1c1c1e] shadow-[0_35px_120px_rgba(0,0,0,0.45)]">
        <WorkspaceHeader
          root={browser.root}
          search={browser.search}
          onRootChange={browser.setRoot}
          onSearchChange={browser.setSearch}
        />

        <section className="grid flex-1 overflow-hidden xl:grid-cols-[280px_minmax(0,1fr)]">
          <WorkspaceSidebar
            paneConfigs={browser.paneConfigs}
            activePane={browser.activePane}
            selectedEcosystems={browser.selectedEcosystems}
            deleteMode={browser.deleteMode}
            statusMetrics={browser.statusMetrics}
            busyAction={browser.busyAction}
            canRunActions={browser.canRunActions}
            onPaneChange={browser.setActivePane}
            onToggleEcosystem={browser.toggleEcosystem}
            onDeleteModeChange={browser.setDeleteMode}
            onScan={browser.handleScan}
            onAnalyze={browser.handleAnalyze}
            onBuildCleanupPlan={browser.handleBuildCleanupPlan}
            onAudit={browser.handleAudit}
            ecosystems={browser.ecosystems}
            deleteModes={browser.deleteModes}
          />

          <div className="grid min-h-0 xl:grid-cols-[minmax(0,1.15fr)_360px]">
            <BrowserPane
              activePaneConfig={browser.activePaneConfig}
              items={browser.currentItems}
              selectedItemId={browser.selectedItemId}
              onSelectItem={browser.setSelectedItemId}
            />

            <WorkspaceInspector
              activePane={browser.activePane}
              activePaneDescription={browser.activePaneConfig.description}
              selectedItem={browser.selectedItem}
              selectedCleanupPaths={browser.selectedCleanupPaths}
              selectedCandidateCount={browser.selectedCandidateCount}
              totalsMetrics={browser.totalsMetrics}
              statusMessage={browser.statusMessage}
              error={browser.error}
              primaryActionLabel={browser.primaryInspectorActionLabel}
              canRunActions={browser.canRunActions}
              busyAction={browser.busyAction}
              onPrimaryAction={browser.handlePrimaryPaneAction}
              onToggleCleanupSelection={browser.handleCleanupSelection}
              onExecuteCleanup={browser.handleExecuteCleanup}
            />
          </div>
        </section>

        <footer className="border-t border-white/8 bg-[#2b2b2e] px-4 py-3">
          <div className="flex flex-col gap-2 text-xs text-slate-400 md:flex-row md:items-center md:justify-between">
            <p>{browser.root || 'No workspace selected'}</p>
            <div className="flex flex-wrap gap-4">
              {browser.footerStats.map((value) => (
                <span key={value}>{value}</span>
              ))}
            </div>
          </div>
        </footer>
      </div>
    </main>
  );
}
