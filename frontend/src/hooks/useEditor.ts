import { useState, useCallback, useEffect, useRef } from "react";
import { localReadFile, localWriteFile, serialReadFile, serialWriteFile, type FileInfo } from "../services/tauri";

const EDITABLE = new Set(["txt", "sub", "ir", "nfc", "json", "conf", "cfg", "ini", "md", "yaml", "yml", "toml", "xml", "csv"]);
const SERIAL_EDITABLE = new Set(["txt", "sub", "ir", "nfc"]);

function ext(name: string) { return name.split(".").pop()?.toLowerCase() || ""; }
export function isEditable(name: string) { return EDITABLE.has(ext(name)); }
export function isSerialEditable(name: string) { return SERIAL_EDITABLE.has(ext(name)); }
export function formatSize(bytes: number): string {
  if (bytes === 0) return "-";
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / 1048576).toFixed(1) + " MB";
}

export interface Tab {
  file: FileInfo;
  content: string;
  original: string;
  dirty: boolean;
  loading: boolean;
  saving: boolean;
}

export interface SearchState {
  query: string;
  replace: string;
  caseSensitive: boolean;
  useRegex: boolean;
  currentMatch: number;
  totalMatches: number;
}

export interface EditorState {
  tabs: Tab[];
  activeTabIndex: number;
  search: SearchState;
  showSearch: boolean;
  autoSave: boolean;
  wordWrap: boolean;
  lineNumbers: boolean;
}

export function useEditor(viewMode: "local" | "serial", mockMode = false) {
  const [state, setState] = useState<EditorState>({
    tabs: [],
    activeTabIndex: -1,
    search: { query: "", replace: "", caseSensitive: false, useRegex: false, currentMatch: 0, totalMatches: 0 },
    showSearch: false,
    autoSave: false,
    wordWrap: true,
    lineNumbers: true,
  });

  const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const viewModeRef = useRef(viewMode);
  const mockModeRef = useRef(mockMode);
  viewModeRef.current = viewMode;
  mockModeRef.current = mockMode;

  // ---- Tab management ----

  const MOCK_FILE_CONTENT: Record<string, string> = {
    "/mock/readme.txt": "# Mock Flipper file system\nThis file simulates an inserted Flipper Zero device.\n\nUse this mode to browse a mock filesystem without a physical device.\n",
    "/mock/apps/subghz.conf": "# Mock subGHz configuration\nenabled=true\nfrequency=433.92\n",
    "/mock/apps/ir_remote.txt": "# Mock IR remote profile\nNAME=TV\nCODE=0x1FE48B7\n",
    "/mock/ext/nfc.bin": "Mock NFC binary content placeholder\n",
  };

  const openFile = useCallback(async (file: FileInfo) => {
    if (!isEditable(file.name)) return;

    setState(prev => {
      // Check if tab already open
      const existingIdx = prev.tabs.findIndex(t => t.file.path === file.path);
      if (existingIdx >= 0) {
        return { ...prev, activeTabIndex: existingIdx };
      }

      const newTab: Tab = { file, content: "", original: "", dirty: false, loading: true, saving: false };
      const newTabs = [...prev.tabs, newTab];
      return { ...prev, tabs: newTabs, activeTabIndex: newTabs.length - 1 };
    });

    // Load content
    try {
      let content: string;
      if (viewMode === "serial" && mockMode) {
        content = MOCK_FILE_CONTENT[file.path] ?? `Mock file content for ${file.name}\n`;
      } else {
        const readFn = viewMode === "serial" ? serialReadFile : localReadFile;
        content = await readFn(file.path);
      }
      setState(prev => {
        const tabs = prev.tabs.map((t, i) =>
          i === prev.activeTabIndex ? { ...t, content, original: content, loading: false, dirty: false } : t
        );
        return { ...prev, tabs };
      });
    } catch (err) {
      setState(prev => {
        const tabs = prev.tabs.map((t, i) =>
          i === prev.activeTabIndex ? { ...t, loading: false } : t
        );
        return { ...prev, tabs };
      });
    }
  }, [viewMode]);

  const closeTab = useCallback((index: number) => {
    setState(prev => {
      const tabs = prev.tabs.filter((_, i) => i !== index);
      let activeTabIndex = prev.activeTabIndex;
      if (index <= activeTabIndex) activeTabIndex = Math.max(0, activeTabIndex - 1);
      if (tabs.length === 0) activeTabIndex = -1;
      return { ...prev, tabs, activeTabIndex };
    });
  }, []);

  const setActiveTab = useCallback((index: number) => {
    setState(prev => ({ ...prev, activeTabIndex: index }));
  }, []);

  // ---- Content editing ----

  const updateContent = useCallback((index: number, content: string) => {
    setState(prev => {
      const tabs = prev.tabs.map((t, i) =>
        i === index ? { ...t, content, dirty: content !== t.original } : t
      );
      return { ...prev, tabs };
    });

    // Auto-save
    if (state.autoSave) {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
      autoSaveTimer.current = setTimeout(() => {
        saveFile(index);
      }, 2000);
    }
  }, [state.autoSave]);

  // ---- Save ----

  const saveFile = useCallback(async (index: number) => {
    setState(prev => {
      const tabs = prev.tabs.map((t, i) =>
        i === index ? { ...t, saving: true } : t
      );
      return { ...prev, tabs };
    });

    const tab = state.tabs[index];
    if (!tab) return;

    try {
      if (viewModeRef.current === "serial" && mockModeRef.current) {
        await new Promise((resolve) => setTimeout(resolve, 100));
      } else {
        const writeFn = viewModeRef.current === "serial" ? serialWriteFile : localWriteFile;
        await writeFn(tab.file.path, tab.content);
      }
      setState(prev => {
        const tabs = prev.tabs.map((t, i) =>
          i === index ? { ...t, original: tab.content, dirty: false, saving: false } : t
        );
        return { ...prev, tabs };
      });
    } catch {
      setState(prev => {
        const tabs = prev.tabs.map((t, i) =>
          i === index ? { ...t, saving: false } : t
        );
        return { ...prev, tabs };
      });
    }
  }, [state.tabs]);

  const saveAll = useCallback(async () => {
    for (let i = 0; i < state.tabs.length; i++) {
      if (state.tabs[i].dirty) await saveFile(i);
    }
  }, [state.tabs, saveFile]);

  // ---- Search & Replace ----

  const toggleSearch = useCallback(() => {
    setState(prev => ({ ...prev, showSearch: !prev.showSearch }));
  }, []);

  const setSearchQuery = useCallback((query: string) => {
    setState(prev => ({ ...prev, search: { ...prev.search, query } }));
  }, []);

  const setReplace = useCallback((replace: string) => {
    setState(prev => ({ ...prev, search: { ...prev.search, replace } }));
  }, []);

  const toggleCaseSensitive = useCallback(() => {
    setState(prev => ({ ...prev, search: { ...prev.search, caseSensitive: !prev.search.caseSensitive } }));
  }, []);

  const findNext = useCallback(() => {
    setState(prev => {
      const total = prev.search.totalMatches;
      if (total === 0) return prev;
      return { ...prev, search: { ...prev.search, currentMatch: (prev.search.currentMatch % total) + 1 } };
    });
  }, []);

  const findPrev = useCallback(() => {
    setState(prev => {
      const total = prev.search.totalMatches;
      if (total === 0) return prev;
      return { ...prev, search: { ...prev.search, currentMatch: ((prev.search.currentMatch - 2 + total) % total) + 1 } };
    });
  }, []);

  const replaceOne = useCallback(() => {
    const { query, replace, currentMatch } = state.search;
    if (!query || currentMatch === 0) return;
    const tab = state.tabs[state.activeTabIndex];
    if (!tab) return;

    const lines = tab.content.split("\n");
    let matchIdx = 0;
    const newLines = lines.map(line => {
      let result = line;
      let searchFrom = 0;
      while (true) {
        const idx = state.search.caseSensitive
          ? result.indexOf(query, searchFrom)
          : result.toLowerCase().indexOf(query.toLowerCase(), searchFrom);
        if (idx === -1) break;
        matchIdx++;
        if (matchIdx === currentMatch) {
          result = result.substring(0, idx) + replace + result.substring(idx + query.length);
          return result;
        }
        searchFrom = idx + query.length;
      }
      return result;
    });

    updateContent(state.activeTabIndex, newLines.join("\n"));
  }, [state.search, state.tabs, state.activeTabIndex, updateContent]);

  const replaceAll = useCallback(() => {
    const { query, replace } = state.search;
    if (!query) return;
    const tab = state.tabs[state.activeTabIndex];
    if (!tab) return;

    const regex = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), state.search.caseSensitive ? "g" : "gi");
    const newContent = tab.content.replace(regex, replace);
    updateContent(state.activeTabIndex, newContent);
  }, [state.search, state.tabs, state.activeTabIndex, updateContent]);

  // ---- Settings ----

  const toggleAutoSave = useCallback(() => {
    setState(prev => ({ ...prev, autoSave: !prev.autoSave }));
  }, []);

  const toggleWordWrap = useCallback(() => {
    setState(prev => ({ ...prev, wordWrap: !prev.wordWrap }));
  }, []);

  const toggleLineNumbers = useCallback(() => {
    setState(prev => ({ ...prev, lineNumbers: !prev.lineNumbers }));
  }, []);

  // ---- Close all ----

  const closeAll = useCallback(() => {
    if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    setState(prev => ({ ...prev, tabs: [], activeTabIndex: -1 }));
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, []);

  return {
    // State
    tabs: state.tabs,
    activeTabIndex: state.activeTabIndex,
    activeTab: state.tabs[state.activeTabIndex] || null,
    selectedFile: state.tabs[state.activeTabIndex]?.file || null,
    showSearch: state.showSearch,
    search: state.search,
    autoSave: state.autoSave,
    wordWrap: state.wordWrap,
    lineNumbers: state.lineNumbers,
    hasDirtyTabs: state.tabs.some(t => t.dirty),
    dirtyCount: state.tabs.filter(t => t.dirty).length,
    dirty: state.tabs.some(t => t.dirty),
    // Tab actions
    openFile,
    closeTab,
    close: closeTab,
    setActiveTab,
    closeAll,
    // Content
    updateContent,
    // Save
    saveFile,
    saveAll,
    // Search
    toggleSearch,
    setSearchQuery,
    setReplace,
    toggleCaseSensitive,
    findNext,
    findPrev,
    replaceOne,
    replaceAll,
    // Settings
    toggleAutoSave,
    toggleWordWrap,
    toggleLineNumbers,
  };
}
