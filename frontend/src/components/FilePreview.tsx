import { type FileInfo } from "../services/tauri";
import { isEditable } from "../hooks/useEditor";
import SubGhzViewer from "./SubGhzViewer";
import IrViewer from "./IrViewer";
import NfcViewer from "./NfcViewer";

interface Props {
  file: FileInfo | null;
  content: string;
  onClose: () => void;
}

export default function FilePreview({ file, content, onClose }: Props) {
  if (!file) return null;

  const ext = file.name.split(".").pop()?.toLowerCase() || "";
  const isFlipperFile = ["sub", "ir", "nfc"].includes(ext);

  return (
    <div className="w-96 bg-gray-800 border-l border-gray-700 flex flex-col overflow-y-auto">
      <div className="px-3 py-2 border-b border-gray-700 flex items-center justify-between shrink-0">
        <h3 className="text-sm font-bold text-gray-300 truncate">{file.name}</h3>
        <button onClick={onClose} className="text-gray-500 hover:text-gray-300 text-xs px-2">✕</button>
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        {ext === "sub" && <SubGhzViewer content={content} fileName={file.name} />}
        {ext === "ir" && <IrViewer content={content} fileName={file.name} />}
        {ext === "nfc" && <NfcViewer content={content} fileName={file.name} />}

        {!isFlipperFile && content && (
          <div className="text-xs text-gray-400 font-mono whitespace-pre-wrap break-all">
            {content.slice(0, 2000)}
            {content.length > 2000 && "\n..."}
          </div>
        )}
      </div>
    </div>
  );
}
