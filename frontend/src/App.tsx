import { useState } from "react";
import { serialDisconnect, createFileFromTemplate, moveFile, type FileInfo } from "./services/tauri";
import { useDirectory } from "./hooks/useDirectory";
import { useEditor } from "./hooks/useEditor";
import { useDragDrop } from "./hooks/useDragDrop";
import FileTable from "./components/FileTable";
import EditorPanel from "./components/EditorPanel";
import NewFileModal from "./components/NewFileModal";
import DevicePanel from "./components/DevicePanel";
import { ToastContainer, showToast } from "./components/ui/Toast";

type ViewMode = "local" | "serial";

export default function App() {
  // --- View mode ---
  const [viewMode, setViewMode] = useState<ViewMode>("local");

  // --- Serial state ---
  const [serialConnected, setSerialConnected] = useState(false);
  const [mockMode, setMockMode] = useState(false);
  const effectiveSerialConnected = serialConnected || mockMode;

  // --- New file modal ---
  const [showNewFile, setShowNewFile] = useState(false);
  const [showReverseEngineer, setShowReverseEngineer] = useState(false);

  // --- Custom hooks ---
  const dir = useDirectory(viewMode, effectiveSerialConnected, mockMode);
  const editor = useEditor(viewMode, mockMode);
  const dnd = useDragDrop(async (file: FileInfo, targetPath: string) => {
    try {
      const dest = targetPath + "/" + file.name;
      await moveFile(file.path, dest);
      dir.refresh();
      showToast(`Moved ${file.name} → ${targetPath}`, "success");
    } catch (err) {
      showToast(err instanceof Error ? err.message : String(err), "error");
    }
  });

  // --- Serial connect/disconnect ---
  const handleDisconnect = async () => {
    try { await serialDisconnect(); } catch { /* ignore */ }
    setSerialConnected(false);
    dir.setCurrentPath("");
    const saved = localStorage.getItem("flipper_root_path");
    if (saved) dir.setCurrentPath(saved);
    editor.closeAll();
    showToast("Disconnected", "info");
  };

  const handleSwitchMode = (mode: ViewMode) => {
    if (mode !== viewMode && serialConnected) handleDisconnect();
    setViewMode(mode);
    if (mode !== "serial") setMockMode(false);
    editor.closeAll();
    dir.setError(null);
  };

  // --- File operations ---
  const handleNavigate = (info: FileInfo) => {
    if (info.is_dir) {
      dir.setCurrentPath(info.path);
      dir.setSearchQuery("");
      editor.closeAll();
    }
  };

  const handleSelectFile = async (file: FileInfo) => {
    editor.openFile(file);
  };

  const handleCreateFile = async (name: string, ext: string) => {
    try {
      const basePath = dir.currentPath + "/" + name;
      await createFileFromTemplate(basePath, ext);
      setShowNewFile(false);
      dir.refresh();
      showToast(`Created ${name}.${ext}`, "success");
    } catch (err) {
      dir.setError(err instanceof Error ? err.message : String(err));
    }
  };

  // --- Render ---
  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center gap-4">
        <h1 className="text-lg font-bold text-emerald-400">Flipper Tool</h1>

        {/* Mode toggle */}
        <div className="flex items-center rounded-2xl bg-gray-700 p-1 gap-1">
          <button
            onClick={() => handleSwitchMode("local")}
            className={`rounded-2xl px-4 py-2 text-sm font-semibold transition border ${viewMode === "local" ? "bg-emerald-600 text-white border-emerald-500" : "border-gray-600 text-gray-200 bg-gray-800 hover:border-gray-500 hover:text-white"}`}
          >
            Local
          </button>
          <button
            onClick={() => handleSwitchMode("serial")}
            className={`rounded-2xl px-4 py-2 text-sm font-semibold transition border ${viewMode === "serial" ? "bg-emerald-600 text-white border-emerald-500" : "border-gray-600 text-gray-200 bg-gray-800 hover:border-gray-500 hover:text-white"}`}
          >
            Serial
          </button>
          {viewMode === "serial" && (
            <button
              onClick={() => setMockMode(!mockMode)}
              className={`rounded-2xl px-4 py-2 text-sm font-semibold transition border ${mockMode ? "bg-purple-600 text-white border-purple-500" : "border-gray-600 text-gray-200 bg-gray-800 hover:border-gray-500 hover:text-white"}`}
            >
              {mockMode ? "Mock On" : "Mock Off"}
            </button>
          )}
        </div>

        {/* Serial controls */}
        {viewMode === "serial" && (
          <div className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col p-3 gap-3">
            <DevicePanel onConnectionChange={setSerialConnected} mockMode={mockMode} />
          </div>
        )}

        {/* Breadcrumb / Path bar */}
        <div className="flex-1 flex items-center gap-2">
          <button
            onClick={dir.goUp}
            className="rounded-2xl border border-gray-600 bg-gray-700 px-4 py-2 text-sm text-gray-200 hover:bg-gray-600"
            title="Go to parent directory"
          >
            Up
          </button>
          <span className="flex-1 rounded-2xl bg-gray-700 px-4 py-2 text-sm font-mono text-gray-300 truncate">
            {viewMode === "serial" && mockMode ? "/mock" : dir.currentPath || "(no folder selected)"}
          </span>
        </div>

        {/* Search + New */}
        <input
          type="text"
          placeholder="Search..."
          value={dir.searchQuery}
          onChange={(e) => dir.setSearchQuery(e.target.value)}
          className="rounded-2xl border border-gray-600 bg-gray-700 px-4 py-2 text-sm text-gray-100 w-52 placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
        />
        <button
          onClick={() => setShowNewFile(!showNewFile)}
          className="rounded-2xl border border-emerald-500 bg-emerald-600 px-4 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-emerald-500"
        >
          + New
        </button>
        <button
          onClick={() => setShowReverseEngineer(!showReverseEngineer)}
          className={`rounded-2xl border px-4 py-2 text-sm font-semibold transition ${showReverseEngineer ? "bg-purple-600 text-white border-purple-500" : "border-gray-600 bg-gray-700 text-gray-200 hover:border-gray-500 hover:text-white"}`}
        >
          Reverse Engineer
        </button>
      </header>

      {/* Error bar */}
      {dir.error && (
        <div className="bg-red-950/95 border-b border-red-700 px-4 py-3 flex flex-col gap-2 text-left">
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-2 text-sm font-semibold text-red-100">
              <span className="inline-flex h-6 w-6 items-center justify-center rounded-full bg-red-700 text-xs">!</span>
              <span>Error loading files</span>
            </div>
            <button onClick={() => dir.setError(null)} className="text-red-300 hover:text-red-100 text-sm">Dismiss</button>
          </div>
          <div className="rounded-lg bg-red-900/80 border border-red-700 p-3 text-xs leading-5 text-red-100 whitespace-pre-wrap">
            {dir.error}
          </div>
          <div className="flex flex-wrap gap-2 text-[11px] text-slate-300">
            <span>Try refreshing or selecting a different folder.</span>
            <button onClick={dir.refresh} className="rounded bg-slate-800 px-2 py-1 text-slate-100 hover:bg-slate-700">Refresh</button>
          </div>
        </div>
      )}

      {/* New file bar */}
      {showNewFile && (
        <NewFileModal
          currentPath={dir.currentPath}
          onClose={() => setShowNewFile(false)}
          onCreate={handleCreateFile}
        />
      )}

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        {/* File browser */}
        <div className="flex-1 overflow-auto">
          {viewMode === "serial" && !serialConnected ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-500 gap-3">
              <span className="text-4xl">🔌</span>
              <span>Select a port and connect to browse Flipper files</span>
              <span className="text-xs text-gray-600">Make sure your Flipper Zero is connected via USB</span>
            </div>
          ) : dir.loading ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              <span className="animate-spin mr-2">⏳</span> Loading...
            </div>
          ) : dir.files.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              {dir.searchQuery ? "No files match your search" : "Empty directory"}
            </div>
          ) : (
            <FileTable
              files={dir.files}
              selectedPath={editor.selectedFile?.path ?? null}
              onSelect={handleSelectFile}
              onOpen={handleNavigate}
              onDragStart={dnd.handleDragStart}
              onDropOnDir={dnd.handleDropOnDir}
            />
          )}
        </div>

        {/* Editor panel with tabs */}
        <EditorPanel
          tabs={editor.tabs}
          activeTabIndex={editor.activeTabIndex}
          showSearch={editor.showSearch}
          search={editor.search}
          autoSave={editor.autoSave}
          wordWrap={editor.wordWrap}
          lineNumbers={editor.lineNumbers}
          hasDirtyTabs={editor.hasDirtyTabs}
          dirtyCount={editor.dirtyCount}
          onContentChange={editor.updateContent}
          onSave={editor.saveFile}
          onSaveAll={editor.saveAll}
          onClose={editor.closeTab}
          onCloseAll={editor.closeAll}
          onSetActive={editor.setActiveTab}
          onToggleSearch={editor.toggleSearch}
          onSetSearchQuery={editor.setSearchQuery}
          onSetReplace={editor.setReplace}
          onToggleCaseSensitive={editor.toggleCaseSensitive}
          onFindNext={editor.findNext}
          onFindPrev={editor.findPrev}
          onReplaceOne={editor.replaceOne}
          onReplaceAll={editor.replaceAll}
          onToggleAutoSave={editor.toggleAutoSave}
          onToggleWordWrap={editor.toggleWordWrap}
          onToggleLineNumbers={editor.toggleLineNumbers}
          viewMode={viewMode}
        />
      </div>

      {/* Footer */}
      <footer className="bg-gray-800 border-t border-gray-700 px-4 py-2 text-xs text-gray-500 flex justify-between">
        <span>{dir.files.length} items</span>
        <span>{viewMode === "serial" ? (mockMode ? "Mock Flipper Active" : serialConnected ? "Flipper Connected" : "Serial Mode") : "Local Mode"}</span>
        {editor.dirty && <span className="text-amber-400">Unsaved changes</span>}
      </footer>

      <ToastContainer />
    </div>
  );
}
