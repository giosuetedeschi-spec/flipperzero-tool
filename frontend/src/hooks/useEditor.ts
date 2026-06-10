import { useState, useCallback } from "react";
import { localReadFile, localWriteFile, serialReadFile, serialWriteFile, type FileInfo } from "../services/tauri";

const EDITABLE = new Set(["txt", "sub", "ir", "nfc", "json", "conf", "cfg", "ini"]);
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

export function useEditor(viewMode: "local" | "serial") {
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null);
  const [content, setContent] = useState("");
  const [original, setOriginal] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dirty = content !== original;

  const openFile = useCallback(async (file: FileInfo) => {
    setSelectedFile(file);
    if (!isEditable(file.name)) { setContent(""); setOriginal(""); return; }
    setLoading(true); setError(null);
    try {
      let text: string;
      if (viewMode === "serial") {
        if (!isSerialEditable(file.name)) {
          setContent("[ Binary or unsupported file type ]"); setOriginal(""); setLoading(false); return;
        }
        text = await serialReadFile(file.path);
      } else {
        text = await localReadFile(file.path);
      }
      setContent(text); setOriginal(text);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally { setLoading(false); }
  }, [viewMode]);

  const saveFile = useCallback(async () => {
    if (!selectedFile) return;
    setSaving(true); setError(null);
    try {
      if (viewMode === "serial") {
        await serialWriteFile(selectedFile.path, content);
      } else {
        await localWriteFile(selectedFile.path, content);
      }
      setOriginal(content);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally { setSaving(false); }
  }, [selectedFile, content, viewMode]);

  const close = useCallback(() => {
    setSelectedFile(null); setContent(""); setOriginal(""); setError(null);
  }, []);

  return { selectedFile, content, setContent, loading, saving, error, dirty, openFile, saveFile, close };
}
