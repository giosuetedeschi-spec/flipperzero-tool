import { useState, useEffect, useCallback, useRef } from "react";
import { listDirectory, findFiles, type FileInfo } from "../services/tauri";

const MOCK_FILE_TREE: Record<string, FileInfo[]> = {
  "/mock": [
    { path: "/mock/apps", name: "apps", size: 0, is_dir: true, modified: "2026-01-01T12:00:00Z" },
    { path: "/mock/ext", name: "ext", size: 0, is_dir: true, modified: "2026-01-01T12:00:00Z" },
    { path: "/mock/readme.txt", name: "readme.txt", size: 128, is_dir: false, modified: "2026-01-01T12:30:00Z" },
  ],
  "/mock/apps": [
    { path: "/mock/apps/subghz.conf", name: "subghz.conf", size: 2048, is_dir: false, modified: "2026-01-01T12:15:00Z" },
    { path: "/mock/apps/ir_remote.txt", name: "ir_remote.txt", size: 512, is_dir: false, modified: "2026-01-01T12:20:00Z" },
  ],
  "/mock/ext": [
    { path: "/mock/ext/nfc.bin", name: "nfc.bin", size: 4096, is_dir: false, modified: "2026-01-01T12:10:00Z" },
  ],
};

export function useDirectory(viewMode: "local" | "serial", serialConnected: boolean, mockMode: boolean) {
  const [currentPath, setCurrentPath] = useState("");
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const cacheRef = useRef<Map<string, FileInfo[]>>(new Map());
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    try {
      const cached = cacheRef.current.get(path);
      if (cached && !searchQuery) {
        setFiles(cached);
        setLoading(false);
        return;
      }
      let entries: FileInfo[];
      if (viewMode === "serial" && mockMode) {
        const normalized = path || "/mock";
        const list = MOCK_FILE_TREE[normalized] ?? [];
        entries = list.filter((item) =>
          !searchQuery || item.name.toLowerCase().includes(searchQuery.toLowerCase())
        );
      } else if (viewMode === "serial") {
        // Lazy import to avoid circular deps
        const { serialListDir } = await import("../services/tauri");
        entries = await serialListDir(path);
      } else {
        entries = await listDirectory(path);
      }
      cacheRef.current.set(path, entries);
      setFiles(entries);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setFiles([]);
    } finally {
      setLoading(false);
    }
  }, [viewMode, searchQuery]);

  // Reload on path/mode change
  useEffect(() => {
    if (serialConnected || viewMode === "local" || mockMode) {
      loadDirectory(currentPath);
    }
  }, [currentPath, viewMode, serialConnected, mockMode, loadDirectory]);

  // Search with debounce
  useEffect(() => {
    if (!searchQuery.trim()) return;
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    searchTimerRef.current = setTimeout(async () => {
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
    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, [searchQuery, currentPath]);

  const refresh = useCallback(() => {
    cacheRef.current.delete(currentPath);
    loadDirectory(currentPath);
  }, [currentPath, loadDirectory]);

  const goUp = useCallback(() => {
    const parts = currentPath.replace(/\\/g, "/").split("/");
    if (parts.length > 1) {
      parts.pop();
      setCurrentPath(parts.join("/") || "/");
      setSearchQuery("");
    }
  }, [currentPath]);

  return {
    currentPath,
    setCurrentPath,
    files,
    loading,
    error,
    setError,
    searchQuery,
    setSearchQuery,
    refresh,
    goUp,
    cache: cacheRef,
  };
}
