import { type FileInfo } from "../services/tauri";
import { isEditable, formatSize, type Tab } from "../hooks/useEditor";
import CodeMirrorEditor from "./editor/CodeMirrorEditor";
import { useState } from "react";

interface Props {
  tabs: Tab[];
  activeTabIndex: number;
  showSearch: boolean;
  search: { query: string; replace: string; caseSensitive: boolean; currentMatch: number; totalMatches: number };
  autoSave: boolean;
  wordWrap: boolean;
  lineNumbers: boolean;
  hasDirtyTabs: boolean;
  dirtyCount: number;
  onContentChange: (index: number, v: string) => void;
  onSave: (index: number) => void;
  onSaveAll: () => void;
  onClose: (index: number) => void;
  onCloseAll: () => void;
  onSetActive: (index: number) => void;
  onToggleSearch: () => void;
  onSetSearchQuery: (q: string) => void;
  onSetReplace: (r: string) => void;
  onToggleCaseSensitive: () => void;
  onFindNext: () => void;
  onFindPrev: () => void;
  onReplaceOne: () => void;
  onReplaceAll: () => void;
  onToggleAutoSave: () => void;
  onToggleWordWrap: () => void;
  onToggleLineNumbers: () => void;
  viewMode: "local" | "serial";
}

function getLanguage(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() || "";
  if (ext === "json") return "json";
  if (ext === "js" || ext === "ts" || ext === "tsx" || ext === "jsx") return "javascript";
  if (ext === "md") return "markdown";
  if (ext === "yaml" || ext === "yml") return "yaml";
  if (ext === "xml") return "xml";
  if (ext === "toml") return "toml";
  return "";
}

function getTabIcon(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() || "";
  switch (ext) {
    case "json": return "{}";
    case "js": case "ts": case "tsx": return "</>";
    case "md": return "Md";
    case "yaml": case "yml": return "Y";
    case "sub": return "S";
    case "ir": return "I";
    case "nfc": return "N";
    default: return "F";
  }
}

export default function EditorPanel({
  tabs, activeTabIndex, showSearch, search, autoSave, wordWrap, lineNumbers,
  hasDirtyTabs, dirtyCount,
  onContentChange, onSave, onSaveAll, onClose, onCloseAll, onSetActive,
  onToggleSearch, onSetSearchQuery, onSetReplace, onToggleCaseSensitive,
  onFindNext, onFindPrev, onReplaceOne, onReplaceAll,
  onToggleAutoSave, onToggleWordWrap, onToggleLineNumbers, viewMode,
}: Props) {
  const [searchFocused, setSearchFocused] = useState(false);
  const activeTab = tabs[activeTabIndex] || null;

  if (tabs.length === 0) return null;

  return (
    <div className="flex flex-col bg-gray-800 border-l border-gray-700" style={{ width: tabs.length > 0 ? "420px" : "0px" }}>
      {/* Tab bar */}
      <div className="flex items-center border-b border-gray-700 bg-gray-850 shrink-0 overflow-x-auto">
        {tabs.map((tab, i) => (
          <div
            key={tab.file.path}
            onClick={() => onSetActive(i)}
            className={`flex items-center gap-1.5 px-3 py-2 text-xs cursor-pointer border-r border-gray-700 shrink-0 max-w-[140px] ${
              i === activeTabIndex ? "bg-gray-900 text-emerald-400" : "text-gray-400 hover:text-gray-200 hover:bg-gray-800"
            }`}
          >
            <span className="text-[10px] font-bold opacity-60">{getTabIcon(tab.file.name)}</span>
            <span className="truncate">{tab.file.name}</span>
            {tab.dirty && <span className="w-2 h-2 rounded-full bg-orange-400 shrink-0" />}
            <button
              onClick={(e) => { e.stopPropagation(); onClose(i); }}
              className="ml-auto text-gray-600 hover:text-gray-300 shrink-0"
            >
              x
            </button>
          </div>
        ))}
        <div className="flex-1" />
        {hasDirtyTabs && (
          <button onClick={onSaveAll} className="px-2 py-1 text-[10px] bg-orange-600 hover:bg-orange-500 text-white rounded mr-1 shrink-0">
            Save All ({dirtyCount})
          </button>
        )}
        <button onClick={onCloseAll} className="px-2 py-1 text-[10px] text-gray-500 hover:text-gray-300 mr-1 shrink-0">
          Close All
        </button>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-1 px-2 py-1 border-b border-gray-700 bg-gray-850 shrink-0">
        {activeTab?.dirty && (
          <span className="px-1.5 py-0.5 text-[9px] font-bold bg-orange-600 text-white rounded">MODIFIED</span>
        )}
        {activeTab?.saving && (
          <span className="px-1.5 py-0.5 text-[9px] font-bold bg-blue-600 text-white rounded">SAVING...</span>
        )}
        <div className="flex-1" />
        <button onClick={onToggleSearch} className={`px-2 py-0.5 text-[10px] rounded ${showSearch ? "bg-emerald-600 text-white" : "bg-gray-700 text-gray-400"}`}>
          Find/Replace
        </button>
        <button onClick={onToggleAutoSave} className={`px-2 py-0.5 text-[10px] rounded ${autoSave ? "bg-emerald-600 text-white" : "bg-gray-700 text-gray-400"}`}>
          Auto-Save
        </button>
        <button onClick={onToggleWordWrap} className={`px-2 py-0.5 text-[10px] rounded ${wordWrap ? "bg-emerald-600 text-white" : "bg-gray-700 text-gray-400"}`}>
          Wrap
        </button>
        <button onClick={onToggleLineNumbers} className={`px-2 py-0.5 text-[10px] rounded ${lineNumbers ? "bg-emerald-600 text-white" : "bg-gray-700 text-gray-400"}`}>
          Lines
        </button>
        <button onClick={() => onSave(activeTabIndex)} disabled={!activeTab?.dirty} className="px-2 py-0.5 text-[10px] bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-600 disabled:text-gray-500 text-white rounded">
          Save
        </button>
      </div>

      {/* Search & Replace bar */}
      {showSearch && (
        <div className="px-2 py-2 border-b border-gray-700 bg-gray-850 shrink-0 space-y-1">
          <div className="flex items-center gap-1">
            <input
              type="text"
              value={search.query}
              onChange={(e) => onSetSearchQuery(e.target.value)}
              onFocus={() => setSearchFocused(true)}
              onBlur={() => setSearchFocused(false)}
              placeholder="Find..."
              className={`flex-1 px-2 py-1 bg-gray-900 border rounded text-xs font-mono text-gray-300 placeholder-gray-600 ${
                searchFocused ? "border-emerald-500" : "border-gray-600"
              }`}
            />
            <span className="text-[10px] text-gray-500">
              {search.totalMatches > 0 ? `${search.currentMatch}/${search.totalMatches}` : ""}
            </span>
            <button onClick={onFindPrev} className="px-1.5 py-1 bg-gray-700 hover:bg-gray-600 rounded text-[10px] text-gray-400">▲</button>
            <button onClick={onFindNext} className="px-1.5 py-1 bg-gray-700 hover:bg-gray-600 rounded text-[10px] text-gray-400">▼</button>
            <button onClick={onToggleCaseSensitive} className={`px-1.5 py-1 text-[10px] rounded font-bold ${search.caseSensitive ? "bg-emerald-600 text-white" : "bg-gray-700 text-gray-400"}`}>
              Aa
            </button>
          </div>
          <div className="flex items-center gap-1">
            <input
              type="text"
              value={search.replace}
              onChange={(e) => onSetReplace(e.target.value)}
              placeholder="Replace..."
              className="flex-1 px-2 py-1 bg-gray-900 border border-gray-600 rounded text-xs font-mono text-gray-300 placeholder-gray-600"
            />
            <button onClick={onReplaceOne} className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-[10px] text-gray-400">Replace</button>
            <button onClick={onReplaceAll} className="px-2 py-1 bg-orange-600 hover:bg-orange-500 rounded text-[10px] text-white">All</button>
          </div>
        </div>
      )}

      {/* Editor area */}
      <div className="flex-1 overflow-hidden">
        {activeTab ? (
          activeTab.loading ? (
            <div className="flex items-center justify-center h-full text-gray-500 text-sm">Loading...</div>
          ) : !isEditable(activeTab.file.name) ? (
            <div className="p-4 space-y-3">
              <h3 className="text-sm font-bold text-gray-300">File Info</h3>
              <div className="text-xs font-mono text-gray-400 break-all">{activeTab.file.path}</div>
              <div className="text-xs text-gray-500">Size: {formatSize(activeTab.file.size)}</div>
              <div className="flex-1" />
              <p className="text-xs text-gray-600 italic">Not editable as text.</p>
            </div>
          ) : (
            <CodeMirrorEditor
              content={activeTab.content}
              onChange={(v: string) => onContentChange(activeTabIndex, v)}
              language={getLanguage(activeTab.file.name)}
              wordWrap={wordWrap}
              lineNumbers={lineNumbers}
              searchQuery={showSearch ? search.query : ""}
              searchCaseSensitive={search.caseSensitive}
              searchCurrentMatch={search.currentMatch}
            />
          )
        ) : (
          <div className="flex items-center justify-center h-full text-gray-500 text-sm">No file open</div>
        )}
      </div>

      {/* Status bar */}
      <div className="flex items-center justify-between px-2 py-1 border-t border-gray-700 bg-gray-850 text-[10px] text-gray-500 shrink-0">
        <span>{activeTab ? `${activeTab.file.name} | ${viewMode}` : ""}</span>
        <span>
          {activeTab && `${activeTab.content.split("\n").length} lines | ${activeTab.content.length} chars`}
          {activeTab?.dirty && " | *"}
          {autoSave && " | auto"}
        </span>
      </div>
    </div>
  );
}
