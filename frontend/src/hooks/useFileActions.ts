import { delete_file, rename_file, copy_file } from "../services/tauri";

export function useFileActions(onRefresh: () => void, onError: (msg: string) => void) {
  const handleDelete = async (path: string) => {
    try {
      await delete_file(path);
      onRefresh();
    } catch (err) {
      onError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRename = async (path: string, newName: string) => {
    try {
      const newPath = await rename_file(path, newName);
      onRefresh();
      return newPath;
    } catch (err) {
      onError(err instanceof Error ? err.message : String(err));
      return null;
    }
  };

  const handleCopy = async (source: string, dest: string) => {
    try {
      await copy_file(source, dest);
      onRefresh();
    } catch (err) {
      onError(err instanceof Error ? err.message : String(err));
    }
  };

  return { handleDelete, handleRename, handleCopy };
}
