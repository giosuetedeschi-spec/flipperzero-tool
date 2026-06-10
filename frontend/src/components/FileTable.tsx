import { useState } from "react";
import { type FileInfo } from "../services/tauri";
import { formatSize } from "../hooks/useEditor";

function getIcon(info: FileInfo): string {
  if (info.is_dir) return "📁";
  const ext = info.name.split(".").pop()?.toLowerCase() || "";
  switch (ext) {
    case "sub": return "📡";
    case "ir": return "🔴";
    case "nfc": return "📶";
    case "txt": return "📝";
    default: return "📄";
  }
}

type SortKey = "name" | "size" | "modified";
type SortDir = "asc" | "desc";

interface Props {
  files: FileInfo[];
  selectedPath: string | null;
  onSelect: (file: FileInfo) => void;
  onOpen: (file: FileInfo) => void;
  onDragStart?: (file: FileInfo, e: React.DragEvent) => void;
  onDropOnDir?: (dir: FileInfo, e: React.DragEvent) => void;
}

export default function FileTable({ files, selectedPath, onSelect, onOpen, onDragStart, onDropOnDir }: Props) {
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir(sortDir === "asc" ? "desc" : "asc");
    else { setSortKey(key); setSortDir("asc"); }
  };

  const sorted = [...files].sort((a, b) => {
    let cmp = 0;
    if (sortKey === "name") cmp = a.name.localeCompare(b.name);
    else if (sortKey === "size") cmp = a.size - b.size;
    else cmp = (a.modified || "").localeCompare(b.modified || "");
    return sortDir === "asc" ? cmp : -cmp;
  });

  const col = (key: SortKey, label: string, width: string) => (
    <th
      className={`px-4 py-2 ${width} cursor-pointer select-none hover:text-gray-200`}
      onClick={() => toggleSort(key)}
    >
      {label} {sortKey === key ? (sortDir === "asc" ? "▲" : "▼") : ""}
    </th>
  );

  if (files.length === 0) {
    return <div className="flex items-center justify-center h-full text-gray-500">Empty directory</div>;
  }

  return (
    <table className="w-full text-sm">
      <thead className="bg-gray-800 sticky top-0">
        <tr className="text-left text-gray-400">
          {col("name", "Name", "")}
          {col("size", "Size", "w-24")}
          {col("modified", "Modified", "w-40")}
        </tr>
      </thead>
      <tbody>
        {sorted.map((file) => (
          <tr
            key={file.path}
            draggable
            onDragStart={(e) => onDragStart?.(file, e)}
            onDragOver={(e) => { e.preventDefault(); if (file.is_dir) e.dataTransfer.dropEffect = "move"; }}
            onDrop={(e) => { if (file.is_dir) onDropOnDir?.(file, e); }}
            onClick={() => onSelect(file)}
            onDoubleClick={() => onOpen(file)}
            className={
              "cursor-pointer border-b border-gray-800 " +
              (selectedPath === file.path ? "bg-emerald-900/40" : "hover:bg-gray-800/60")
            }
          >
            <td className="px-4 py-2 text-center">{getIcon(file)}</td>
            <td className="px-4 py-2 font-mono">{file.name}</td>
            <td className="px-4 py-2 text-gray-400">{formatSize(file.size)}</td>
            <td className="px-4 py-2 text-gray-400">
              {file.modified ? new Date(parseInt(file.modified) * 1000).toLocaleDateString() : "-"}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
