import { useState, useEffect } from "react";
import { parserParseNfcStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

export default function NfcViewer({ content, fileName }: Props) {
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    parserParseNfcStruct(content)
      .then((result: any) => setData(result.fields?.[0] || result))
      .catch((e: any) => setError(String(e)));
  }, [content]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  // Determine card color based on device type
  const cardColor = data.device_type?.includes("Mifare Classic")
    ? "from-blue-900 to-blue-700"
    : data.device_type?.includes("Mifare Ultralight")
    ? "from-green-900 to-green-700"
    : "from-gray-700 to-gray-600";

  return (
    <div className="p-3 bg-gray-800 rounded-lg space-y-3">
      <h4 className="text-sm font-bold text-purple-400">📶 NFC File</h4>

      {/* Card visualization */}
      <div className={`bg-gradient-to-br ${cardColor} rounded-lg p-4 text-center`}>
        <div className="text-2xl mb-1">💳</div>
        <div className="text-white font-bold text-sm">{data.device_type || "Unknown"}</div>
        <div className="text-white/70 text-xs font-mono mt-1">{data.uid || "No UID"}</div>
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">UID</span>
          <div className="text-white font-mono text-[10px] break-all">{data.uid || "—"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">ATQA</span>
          <div className="text-yellow-400 font-mono">{data.atqa || "—"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">SAK</span>
          <div className="text-green-400 font-mono">{data.sak ? `0x${data.sak.toString(16).padStart(2, '0').toUpperCase()}` : "—"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Version</span>
          <div className="text-gray-300 font-mono">{data.version || "—"}</div>
        </div>
      </div>

      {data.sectors && data.sectors.length > 0 && (
        <div className="space-y-1">
          <span className="text-gray-400 text-xs">Sectors ({data.sectors.length})</span>
          <div className="max-h-32 overflow-y-auto space-y-1">
            {data.sectors.map((sector: any, i: number) => (
              <div key={i} className="bg-gray-700/50 rounded p-2 text-xs">
                <span className="text-gray-300">Sector {sector.index}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
