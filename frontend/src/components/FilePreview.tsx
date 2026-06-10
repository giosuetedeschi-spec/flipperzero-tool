import { useState, useEffect } from "react";
import { parser_parse_sub, parser_parse_ir, parser_parse_nfc, type FileInfo } from "../services/tauri";
import { isEditable } from "../hooks/useEditor";

interface Props {
  file: FileInfo | null;
  content: string;
  onClose: () => void;
}

interface ParsedData {
  file_type: string;
  fields: Array<{ key: string; value: unknown }>;
}

export default function FilePreview({ file, content, onClose }: Props) {
  const [parsed, setParsed] = useState<ParsedData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!file || !isEditable(file.name)) {
      setParsed(null);
      return;
    }

    const ext = file.name.split(".").pop()?.toLowerCase() || "";
    const parse = async () => {
      try {
        setError(null);
        let result: ParsedData;
        if (ext === "sub") {
          result = await parser_parse_sub(content);
        } else if (ext === "ir") {
          result = await parser_parse_ir(content);
        } else if (ext === "nfc") {
          result = await parser_parse_nfc(content);
        } else {
          setParsed(null);
          return;
        }
        setParsed(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    };
    parse();
  }, [file, content]);

  if (!file) return null;
  const ext = file.name.split(".").pop()?.toLowerCase() || "";

  return (
    <div className="w-80 bg-gray-800 border-l border-gray-700 flex flex-col">
      <div className="px-3 py-2 border-b border-gray-700 flex items-center justify-between">
        <h3 className="text-sm font-bold text-gray-300 truncate">{file.name}</h3>
        <button onClick={onClose} className="text-gray-500 hover:text-gray-300 text-xs">X</button>
      </div>

      <div className="flex-1 overflow-auto p-3 space-y-2">
        {error && (
          <div className="bg-red-900/40 border border-red-800 rounded px-2 py-1 text-xs text-red-300">
            {error}
          </div>
        )}

        {parsed && (
          <>
            <div className="text-xs text-emerald-400 font-medium uppercase tracking-wide">
              {parsed.file_type} file
            </div>
            {parsed.fields.map((f, i) => (
              <div key={i} className="flex justify-between text-xs">
                <span className="text-gray-400 font-medium">{f.key}</span>
                <span className="text-gray-200 font-mono ml-2 truncate max-w-32" title={String(f.value)}>
                  {typeof f.value === "object" ? JSON.stringify(f.value).slice(0, 40) : String(f.value)}
                </span>
              </div>
            ))}
          </>
        )}

        {(ext === "txt" || !parsed) && content && (
          <div className="text-xs text-gray-400 font-mono whitespace-pre-wrap break-all max-h-60 overflow-auto">
            {content.slice(0, 500)}
            {content.length > 500 && "..."}
          </div>
        )}
      </div>
    </div>
  );
}
