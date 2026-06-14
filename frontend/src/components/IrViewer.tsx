import { useState, useEffect } from "react";
import { parserParseIrStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

export default function IrViewer({ content, fileName }: Props) {
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    parserParseIrStruct(content)
      .then((result: any) => setData(result.fields?.[0] || result))
      .catch((e: any) => setError(String(e)));
  }, [content]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  return (
    <div className="p-3 bg-gray-800 rounded-lg space-y-3">
      <h4 className="text-sm font-bold text-orange-400">🔴 Infrared File</h4>

      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Protocol</span>
          <div className="text-yellow-400 font-mono">{data.protocol || "Unknown"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Address</span>
          <div className="text-blue-400 font-mono">{data.address || "—"}</div>
        </div>
        <div className="col-span-2 bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Command</span>
          <div className="text-green-400 font-mono">{data.command || "—"}</div>
        </div>
      </div>

      {data.buttons && data.buttons.length > 0 && (
        <div className="space-y-1">
          <span className="text-gray-400 text-xs">Buttons ({data.buttons.length})</span>
          <div className="max-h-32 overflow-y-auto space-y-1">
            {data.buttons.map((btn: any, i: number) => (
              <div key={i} className="bg-gray-700/50 rounded p-2 text-xs flex justify-between">
                <span className="text-gray-300">{btn.name}</span>
                <span className="text-gray-500 font-mono">
                  {btn.protocol} {btn.address} {btn.command}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {data.is_raw && (
        <div className="bg-orange-900/30 rounded p-2 text-orange-400 text-xs">
          ⚡ RAW signal detected
        </div>
      )}
    </div>
  );
}
