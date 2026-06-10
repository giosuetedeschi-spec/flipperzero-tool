import { useState, useEffect, useCallback } from "react";
import { listDirectory, serialListPorts, serialConnect, serialDisconnect, serialIsConnected, createFileFromTemplate, type FileInfo, type PortInfo } from "./services/tauri";
import { useDirectory } from "./hooks/useDirectory";
import { useEditor } from "./hooks/useEditor";
import FileTable from "./components/FileTable";
import EditorPanel from "./components/EditorPanel";
import NewFileModal from "./components/NewFileModal";
import SerialPanel from "./components/SerialPanel";
import { ToastContainer, showToast } from "./components/ui/Toast";

type ViewMode = "local" | "serial";

export default function App() {
  // --- View mode ---
  const [viewMode, setViewMode] = useState<ViewMode>("local");

  // --- Serial state ---
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [serialConnected, setSerialConnected] = useState(false);
  const [serialError, setSerialError] = useState<string | null>(null);

  // --- New file modal ---
  const [showNewFile, setShowNewFile] = useState(false);

  // --- Custom hooks ---
  const dir = useDirectory(viewMode, serialConnected);
  const editor = useEditor(viewMode);

  // --- Serial port discovery ---
  const refreshPorts = useCallback(async () => {
    try {
      const p = await serialListPorts();
      setPorts(p);
      setSerialError(null);
    } catch (err) {
      setSerialError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    if (viewMode === "serial") {
      refreshPorts();
      // Auto-refresh every 5s
      const interval = setInterval(refreshPorts, 5000);
      return () => clearInterval(interval);
    }
  }, [viewMode, refreshPorts]);

  // --- Serial connect/disconnect ---
  const handleConnect = async () => {
    if (!selectedPort) { setSerialError("Select a port first"); return; }
    setSerialError(null);
    try {
      await serialConnect(selectedPort);
      setSerialConnected(true);
      dir.setCurrentPath("/ext");
      showToast("Connected to Flipper", "success");
    } catch (err) {
      setSerialError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDisconnect = async () => {
    try { await serialDisconnect(); } catch { /* ignore */ }
    setSerialConnected(false);
    dir.setCurrentPath("");
    const saved = localStorage.getItem("flipper_root_path");
    if (saved) dir.setCurrentPath(saved);
    editor.close();
    setSelectedPort("");
    showToast("Disconnected", "info");
  };

  const handleSwitchMode = (mode: ViewMode) => {
    if (mode !== viewMode && serialConnected) handleDisconnect();
    setViewMode(mode);
    editor.close();
    dir.setError(null);
  };

  // --- File operations ---
  const handleNavigate = (info: FileInfo) => {
    if (info.is_dir) {
      dir.setCurrentPath(info.path);
      dir.setSearchQuery("");
      editor.close();
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

  const handleDeleteFile = async (path: string) => {
    try {
      const { delete_file } = await import("./services/tauri");
      await delete_file(path);
      dir.refresh();
      showToast("File deleted", "success");
    } catch (err) {
      showToast(err instanceof Error ? err.message : String(err), "error");
    }
  };

  // --- Render ---
  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center gap-4">
        <h1 className="text-lg font-bold text-emerald-400">Flipper Tool</h1>

        {/* Mode toggle */}
        <div className="flex items-center bg-gray-700 rounded-lg p-0.5">
          <button
            onClick={() => handleSwitchMode("local")}
            className={`px-3 py-1 rounded text-sm font-medium transition ${viewMode === "local" ? "bg-emerald-600 text-white" : "text-gray-400 hover:text-gray-200"}`}
          >
            Local
          </button>
          <button
            onClick={() => handleSwitchMode("serial")}
            className={`px-3 py-1 rounded text-sm font-medium transition ${viewMode === "serial" ? "bg-emerald-600 text-white" : "text-gray-400 hover:text-gray-200"}`}
          >
            Serial
          </button>
        </div>

        {/* Serial controls */}
        {viewMode === "serial" && (
          <SerialPanel
            ports={ports}
            selectedPort={selectedPort}
            connected={serialConnected}
            error={serialError}
            onSelectPort={setSelectedPort}
            onRefresh={refreshPorts}
            onConnect={handleConnect}
            onDisconnect={handleDisconnect}
            onCloseError={() => setSerialError(null)}
          />
        )}

        {/* Breadcrumb / Path bar */}
        <div className="flex-1 flex items-center gap-2">
          <button
            onClick={dir.goUp}
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
            title="Go to parent directory"
          >
            Up
          </button>
          <span className="flex-1 px-3 py-1 bg-gray-700 rounded text-sm font-mono text-gray-300 truncate">
            {dir.currentPath || "(no folder selected)"}
          </span>
        </div>

        {/* Search + New */}
        <input
          type="text"
          placeholder="Search..."
          value={dir.searchQuery}
          onChange={(e) => dir.setSearchQuery(e.target.value)}
          className="px-3 py-1 bg-gray-700 rounded text-sm w-48 placeholder-gray-500"
        />
        <button
          onClick={() => setShowNewFile(!showNewFile)}
          className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-medium"
        >
          + New
        </button>
      </header>

      {/* Error bar */}
      {dir.error && (
        <div className="bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center justify-between">
          <span className="text-red-200 text-sm">{dir.error}</span>
          <button onClick={() => dir.setError(null)} className="text-red-300 hover:text-red-100 text-sm">X</button>
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
            />
          )}
        </div>

        {/* Editor panel */}
        <EditorPanel
          file={editor.selectedFile}
          content={editor.content}
          original={editor.original}
          loading={editor.loading}
          saving={editor.saving}
          error={editor.error}
          viewMode={viewMode}
          onContentChange={editor.setContent}
          onSave={editor.saveFile}
          onClose={editor.close}
        />
      </div>

      {/* Footer */}
      <footer className="bg-gray-800 border-t border-gray-700 px-4 py-2 text-xs text-gray-500 flex justify-between">
        <span>{dir.files.length} items</span>
        <span>{viewMode === "serial" ? (serialConnected ? "Flipper Connected" : "Serial Mode") : "Mock SD"}</span>
        {editor.dirty && <span className="text-amber-400">Unsaved changes</span>}
      </footer>

      <ToastContainer />
    </div>
  );
}
