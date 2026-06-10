import { useState, useEffect, useCallback } from "react";
import {
  listDirectory,
  findFiles,
  createFileFromTemplate,
  localReadFile,
  localWriteFile,
  serialListPorts,
  serialConnect,
  serialDisconnect,
  serialReadFile,
  serialWriteFile,
  serialIsConnected,
  type FileInfo,
  type PortInfo,
} from "./services/tauri";

const MOCK_ROOT = "C:/Cose Nuove/Code/flipperzero-tool/.flipper_mock";

// File types that support text editing
const EDITABLE_EXTENSIONS = new Set(["txt", "sub", "ir", "nfc", "json", "conf", "cfg", "ini"]);
const SERIAL_FILE_TYPES = new Set(["txt", "sub", "ir", "nfc"]);

function getExtension(name: string): string {
  const parts = name.split(".");
  return parts.length > 1 ? parts.pop()!.toLowerCase() : "";
}

function isEditable(name: string): boolean {
  return EDITABLE_EXTENSIONS.has(getExtension(name));
}

function isSerialEditable(name: string): boolean {
  return SERIAL_FILE_TYPES.has(getExtension(name));
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "-";
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function getIcon(info: FileInfo): string {
  if (info.is_dir) return "📁";
  const ext = getExtension(info.name);
  switch (ext) {
    case "sub": return "📡";
    case "ir": return "🔴";
    case "nfc": return "📶";
    case "txt": return "📝";
    default: return "📄";
  }
}

type ViewMode = "local" | "serial";

export default function App() {
  // --- View mode ---
  const [viewMode, setViewMode] = useState<ViewMode>("local");

  // --- Serial state ---
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>("");
  const [serialConnected, setSerialConnected] = useState(false);
  const [serialError, setSerialError] = useState<string | null>(null);

  // --- File browser state ---
  const [currentPath, setCurrentPath] = useState(MOCK_ROOT);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null);
  const [showNewFile, setShowNewFile] = useState(false);
  const [newFileName, setNewFileName] = useState("");
  const [newFileExt, setNewFileExt] = useState("sub");

  // --- Editor state ---
  const [editContent, setEditContent] = useState<string>("");
  const [originalContent, setOriginalContent] = useState<string>("");
  const [editorLoading, setEditorLoading] = useState(false);
  const [editorSaving, setEditorSaving] = useState(false);
  const [editorError, setEditorError] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);

  // --- Discover serial ports ---
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
    }
  }, [viewMode, refreshPorts]);

  // --- Load directory ---
  const loadDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    try {
      let entries: FileInfo[];
      if (viewMode === "serial") {
        entries = await serialListDir(path);
      } else {
        entries = await listDirectory(path);
      }
      setFiles(entries);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setFiles([]);
    } finally {
      setLoading(false);
    }
  }, [viewMode]);

  useEffect(() => {
    if (serialConnected || viewMode === "local") {
      loadDirectory(currentPath);
    }
  }, [currentPath, viewMode, serialConnected, loadDirectory]);

  // --- Search ---
  useEffect(() => {
    if (!searchQuery.trim()) return;
    const timer = setTimeout(async () => {
      setLoading(true);
      setError(null);
      try {
        const results = await findFiles(currentPath, searchQuery);
        setFiles(results);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery, currentPath]);

  // --- Serial connect/disconnect ---
  const handleConnect = async () => {
    if (!selectedPort) {
      setSerialError("Select a port first");
      return;
    }
    setSerialError(null);
    try {
      await serialConnect(selectedPort);
      setSerialConnected(true);
      setCurrentPath("/ext"); // Standard Flipper external storage root
    } catch (err) {
      setSerialError(err instanceof Error ? err.message : String(err));
      setSerialConnected(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await serialDisconnect();
    } catch {
      // ignore
    }
    setSerialConnected(false);
    setCurrentPath(MOCK_ROOT);
    setSelectedFile(null);
    setEditContent("");
    setOriginalContent("");
    setDirty(false);
  };

  const handleSwitchMode = (mode: ViewMode) => {
    if (mode === "serial" && serialConnected) {
      handleDisconnect();
    }
    setViewMode(mode);
    setSelectedFile(null);
    setEditContent("");
    setOriginalContent("");
    setDirty(false);
    setError(null);
  };

  // --- Navigation ---
  const handleNavigate = (info: FileInfo) => {
    if (info.is_dir) {
      setCurrentPath(info.path);
      setSearchQuery("");
      setSelectedFile(null);
      setEditContent("");
      setOriginalContent("");
      setDirty(false);
    }
  };

  const handleGoUp = () => {
    const parts = currentPath.replace(/\\/g, "/").split("/");
    if (parts.length > 1) {
      parts.pop();
      const parent = parts.join("/") || "/";
      setCurrentPath(parent);
      setSearchQuery("");
      setSelectedFile(null);
      setEditContent("");
      setOriginalContent("");
      setDirty(false);
    }
  };

  const handleCreateFile = async () => {
    if (!newFileName.trim()) return;
    setError(null);
    try {
      const basePath = currentPath + "/" + newFileName.trim();
      await createFileFromTemplate(basePath, newFileExt);
      setShowNewFile(false);
      setNewFileName("");
      await loadDirectory(currentPath);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  // --- File open / editor ---
  const handleSelectFile = async (file: FileInfo) => {
    setSelectedFile(file);
    if (!isEditable(file.name)) {
      setEditContent("");
      setOriginalContent("");
      setDirty(false);
      return;
    }

    setEditorLoading(true);
    setEditorError(null);
    try {
      let content: string;
      if (viewMode === "serial") {
        if (!isSerialEditable(file.name)) {
          setEditContent("[ Binary or unsupported file type for serial editing ]");
          setOriginalContent("");
          setEditorLoading(false);
          return;
        }
        content = await serialReadFile(file.path);
      } else {
        content = await localReadFile(file.path);
      }
      setEditContent(content);
      setOriginalContent(content);
      setDirty(false);
    } catch (err) {
      setEditorError(err instanceof Error ? err.message : String(err));
      setEditContent("");
      setOriginalContent("");
    } finally {
      setEditorLoading(false);
    }
  };

  const handleSaveFile = async () => {
    if (!selectedFile) return;
    setEditorSaving(true);
    setEditorError(null);
    try {
      if (viewMode === "serial") {
        await serialWriteFile(selectedFile.path, editContent);
      } else {
        await localWriteFile(selectedFile.path, editContent);
      }
      setOriginalContent(editContent);
      setDirty(false);
    } catch (err) {
      setEditorError(err instanceof Error ? err.message : String(err));
    } finally {
      setEditorSaving(false);
    }
  };

  const handleContentChange = (value: string) => {
    setEditContent(value);
    setDirty(value !== originalContent);
  };

  // --- Render ---
  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center gap-4">
        <h1 className="text-lg font-bold text-emerald-400">🐬 Flipper Tool</h1>

        {/* Mode toggle */}
        <div className="flex items-center bg-gray-700 rounded-lg p-0.5">
          <button
            onClick={() => handleSwitchMode("local")}
            className={`px-3 py-1 rounded text-sm font-medium transition ${
              viewMode === "local" ? "bg-emerald-600 text-white" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            📂 Local
          </button>
          <button
            onClick={() => handleSwitchMode("serial")}
            className={`px-3 py-1 rounded text-sm font-medium transition ${
              viewMode === "serial" ? "bg-emerald-600 text-white" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            🔌 Serial
          </button>
        </div>

        {/* Serial controls */}
        {viewMode === "serial" && (
          <div className="flex items-center gap-2">
            <select
              value={selectedPort}
              onChange={(e) => setSelectedPort(e.target.value)}
              className="px-2 py-1 bg-gray-700 rounded text-sm min-w-48 disabled:opacity-50"
              disabled={serialConnected}
            >
              <option value="">-- Select Port --</option>
              {ports.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name} ({p.description || p.port_type})
                </option>
              ))}
            </select>
            {!serialConnected ? (
              <>
                <button
                  onClick={refreshPorts}
                  className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
                  title="Refresh ports"
                >
                  🔄
                </button>
                <button
                  onClick={handleConnect}
                  className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-medium"
                >
                  Connect
                </button>
              </>
            ) : (
              <button
                onClick={handleDisconnect}
                className="px-3 py-1 bg-red-600 hover:bg-red-500 rounded text-sm font-medium"
              >
                Disconnect
              </button>
            )}
            {serialConnected && (
              <span className="text-xs text-emerald-400 flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                Connected
              </span>
            )}
          </div>
        )}

        {/* Path bar */}
        <div className="flex-1 flex items-center gap-2">
          <button
            onClick={handleGoUp}
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
          >
            ⬆ Up
          </button>
          <input
            type="text"
            value={currentPath}
            readOnly
            className="flex-1 px-3 py-1 bg-gray-700 rounded text-sm font-mono text-gray-300"
          />
        </div>

        {/* Search + New */}
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="Search..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="px-3 py-1 bg-gray-700 rounded text-sm w-48 placeholder-gray-500"
          />
          <button
            onClick={() => setShowNewFile(!showNewFile)}
            className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-medium"
          >
            + New
          </button>
        </div>
      </header>

      {/* Serial error */}
      {serialError && (
        <div className="bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center justify-between">
          <span className="text-red-200 text-sm">⚠️ Serial: {serialError}</span>
          <button onClick={() => setSerialError(null)} className="text-red-300 hover:text-red-100 text-sm">✕</button>
        </div>
      )}

      {/* General error */}
      {error && (
        <div className="bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center justify-between">
          <span className="text-red-200 text-sm">⚠️ {error}</span>
          <button onClick={() => setError(null)} className="text-red-300 hover:text-red-100 text-sm">✕</button>
        </div>
      )}

      {/* New file bar */}
      {showNewFile && (
        <div className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center gap-3">
          <span className="text-sm text-gray-400">New file:</span>
          <input
            type="text"
            placeholder="filename"
            value={newFileName}
            onChange={(e) => setNewFileName(e.target.value)}
            className="px-3 py-1 bg-gray-700 rounded text-sm w-48 placeholder-gray-500"
            autoFocus
          />
          <select
            value={newFileExt}
            onChange={(e) => setNewFileExt(e.target.value)}
            className="px-2 py-1 bg-gray-700 rounded text-sm"
          >
            <option value="sub">.sub (Sub-GHz)</option>
            <option value="ir">.ir (Infrared)</option>
            <option value="nfc">.nfc (NFC)</option>
            <option value="txt">.txt (BadUSB)</option>
          </select>
          <button
            onClick={handleCreateFile}
            className="px-4 py-1 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-medium"
          >
            Create
          </button>
          <button
            onClick={() => setShowNewFile(false)}
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
          >
            Cancel
          </button>
        </div>
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
          ) : loading ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              <span className="animate-spin mr-2">⏳</span> Loading...
            </div>
          ) : files.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              {searchQuery ? "No files match your search" : "Empty directory"}
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead className="bg-gray-800 sticky top-0">
                <tr className="text-left text-gray-400">
                  <th className="px-4 py-2 w-8"></th>
                  <th className="px-4 py-2">Name</th>
                  <th className="px-4 py-2 w-24">Size</th>
                  <th className="px-4 py-2 w-40">Modified</th>
                </tr>
              </thead>
              <tbody>
                {files.map((file) => (
                  <tr
                    key={file.path}
                    onClick={() => handleSelectFile(file)}
                    onDoubleClick={() => handleNavigate(file)}
                    className={
                      "cursor-pointer border-b border-gray-800 " +
                      (selectedFile?.path === file.path
                        ? "bg-emerald-900/40"
                        : "hover:bg-gray-800/60")
                    }
                  >
                    <td className="px-4 py-2 text-center">{getIcon(file)}</td>
                    <td className="px-4 py-2 font-mono">{file.name}</td>
                    <td className="px-4 py-2 text-gray-400">{formatSize(file.size)}</td>
                    <td className="px-4 py-2 text-gray-400">
                      {file.modified
                        ? new Date(parseInt(file.modified) * 1000).toLocaleDateString()
                        : "-"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Editor panel */}
        {selectedFile && isEditable(selectedFile.name) && (
          <div className="w-96 bg-gray-800 border-l border-gray-700 flex flex-col">
            {/* Editor header */}
            <div className="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
              <div className="flex-1 min-w-0">
                <h3 className="text-sm font-bold text-gray-300 truncate">{selectedFile.name}</h3>
                <p className="text-xs text-gray-500 font-mono truncate">{selectedFile.path}</p>
              </div>
              <div className="flex items-center gap-2 ml-2">
                {dirty && (
                  <span className="text-xs text-amber-400" title="Unsaved changes">●</span>
                )}
                {editorSaving && (
                  <span className="text-xs text-gray-400 animate-pulse">Saving...</span>
                )}
                <button
                  onClick={handleSaveFile}
                  disabled={!dirty || editorSaving || editorLoading}
                  className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed rounded text-sm font-medium transition"
                >
                  💾 Save
                </button>
              </div>
            </div>

            {/* Editor error */}
            {editorError && (
              <div className="bg-red-900/60 border-b border-red-800 px-3 py-2 text-xs text-red-300">
                ⚠️ {editorError}
              </div>
            )}

            {/* Editor body */}
            <div className="flex-1 overflow-hidden">
              {editorLoading ? (
                <div className="flex items-center justify-center h-full text-gray-500">
                  <span className="animate-spin mr-2">⏳</span> Loading file...
                </div>
              ) : (
                <textarea
                  value={editContent}
                  onChange={(e) => handleContentChange(e.target.value)}
                  className="w-full h-full bg-gray-900 text-gray-100 font-mono text-sm p-4 resize-none focus:outline-none focus:ring-2 focus:ring-emerald-600/50 border-none"
                  spellCheck={false}
                  placeholder="File content..."
                ></textarea>
              )}
            </div>

            {/* Status bar */}
            <div className="px-4 py-2 border-t border-gray-700 text-xs text-gray-500 flex justify-between">
              <span>{editContent.length} chars</span>
              <span>{editContent.split("\n").length} lines</span>
              <span>{viewMode === "serial" ? "Serial" : "Local"}</span>
            </div>
          </div>
        )}

        {/* Non-editable file info panel */}
        {selectedFile && !isEditable(selectedFile.name) && (
          <div className="w-72 bg-gray-800 border-l border-gray-700 p-4 flex flex-col gap-3">
            <h3 className="text-sm font-bold text-gray-300">File Info</h3>
            <div className="text-xs font-mono text-gray-400 break-all">{selectedFile.path}</div>
            <div className="text-xs text-gray-500">Size: {formatSize(selectedFile.size)}</div>
            <div className="text-xs text-gray-500">Type: {getExtension(selectedFile.name) || "unknown"}</div>
            <div className="flex-1" />
            <p className="text-xs text-gray-600 italic">This file type is not editable as text.</p>
          </div>
        )}
      </div>

      {/* Footer */}
      <footer className="bg-gray-800 border-t border-gray-700 px-4 py-2 text-xs text-gray-500 flex justify-between">
        <span>{files.length} items</span>
        <span>{viewMode === "serial" ? (serialConnected ? "🔌 Flipper Connected" : "🔌 Serial Mode") : "📂 Mock SD"}</span>
        {dirty && <span className="text-amber-400">Unsaved changes</span>}
      </footer>
    </div>
  );
}
