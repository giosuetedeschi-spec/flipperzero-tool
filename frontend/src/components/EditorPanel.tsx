import { type FileInfo } from "../services/tauri";
import { isEditable, formatSize } from "../hooks/useEditor";

interface Props {
  file: FileInfo | null;
  content: string;
  original: string;
  loading: boolean;
  saving: boolean;
  error: string | null;
  viewMode: "local" | "serial";
  onContentChange: (v: string) => void;
  onSave: () => void;
  onClose: () => void;
}

export default function EditorPanel({ file, content, original, loading, saving, error, viewMode, onContentChange, onSave, onClose }: Props) {
  if (!file) return null;
  const editable = isEditable(file.name);
  const dirty = content !== original;

  if (!editable) {
    return (
      <div className="w-72 bg-gray-800 border-l border-gray-700 p-4 flex flex-col gap-3">
        <h3 className="text-sm font-bold text-gray-300">File Info</h3>
        <div className="text-xs font-mono text-gray-400 break-all">{file.path}</div>
        <div className="text-xs text-gray-500">Size: {formatSize(file.size)}</div>
        <div className="flex-1" />
        <p className="text-xs text-gray-600 italic">Not editable as text.</p>
      </div>
    );
  }

  return (
    <div className="w-96 bg-gray-800 border-l border-gray-700 flex flex-col">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-700 flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <h3 className="text-sm font-bold text-gray-300 truncate">{file.name}</h3>
          <p className="text-xs text-gray-500 font-mono truncate">{file.path}</p>
        </div>
        <div className="flex items-center gap-2 ml-2">
          {dirty && <span className="text-xs text-amber-400" title="Unsaved">●</span>}
          {saving && <span className="text-xs text-gray-400 animate-pulse">Saving...</span>}
          <button
            onClick={onSave}
            disabled={!dirty || saving || loading}
            className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed rounded text-sm font-medium transition"
          >
            💾 Save
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-900/60 border-b border-red-800 px-3 py-2 text-xs text-red-300">
          ⚠️ {error}
        </div>
      )}

      <div className="flex-1 overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            <span className="animate-spin mr-2">⏳</span> Loading...
          </div>
        ) : (
          <textarea
            value={content}
            onChange={(e) => onContentChange(e.target.value)}
            className="w-full h-full bg-gray-900 text-gray-100 font-mono text-sm p-4 resize-none focus:outline-none focus:ring-2 focus:ring-emerald-600/50 border-none"
            spellCheck={false}
            placeholder="File content..."
          />
        )}
      </div>

      <div className="px-4 py-2 border-t border-gray-700 text-xs text-gray-500 flex justify-between">
        <span>{content.length} chars</span>
        <span>{content.split("\n").length} lines</span>
        <span>{viewMode === "serial" ? "Serial" : "Local"}</span>
      </div>
    </div>
  );
}
