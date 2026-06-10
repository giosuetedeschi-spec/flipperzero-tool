import { useState, useCallback, useRef } from "react";
import { type FileInfo } from "../services/tauri";

interface DragState {
  isDragging: boolean;
  draggedFile: FileInfo | null;
}

export function useDragDrop(onDrop: (file: FileInfo, targetPath: string) => void) {
  const [dragState, setDragState] = useState<DragState>({ isDragging: false, draggedFile: null });
  const dragRef = useRef<HTMLDivElement | null>(null);

  const handleDragStart = useCallback((file: FileInfo, e: React.DragEvent) => {
    e.dataTransfer.setData("application/json", JSON.stringify(file));
    e.dataTransfer.effectAllowed = "move";
    setDragState({ isDragging: true, draggedFile: file });
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  }, []);

  const handleDragEnd = useCallback(() => {
    setDragState({ isDragging: false, draggedFile: null });
  }, []);

  const handleDropOnDir = useCallback((targetDir: FileInfo, e: React.DragEvent) => {
    e.preventDefault();
    if (!targetDir.is_dir) return;
    try {
      const file: FileInfo = JSON.parse(e.dataTransfer.getData("application/json"));
      onDrop(file, targetDir.path);
    } catch { /* ignore */ }
    setDragState({ isDragging: false, draggedFile: null });
  }, [onDrop]);

  return {
    isDragging: dragState.isDragging,
    handleDragStart,
    handleDragOver,
    handleDragEnd,
    handleDropOnDir,
  };
}
