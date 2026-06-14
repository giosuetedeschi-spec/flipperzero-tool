import { useState, useEffect, useMemo } from "react";
import { parserParseSubStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

interface SubGhzData {
  filetype: string;
  version: number;
  frequency: number;
  frequency_display: string;
  preset: string;
  protocol: string;
  bit: number | null;
  key: string;
  is_raw: boolean;
}

export default function SubGhzViewer({ content, fileName }: Props) {
  const [data, setData] = useState<SubGhzData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    parserParseSubStruct(content)
      .then((result: any) => {
        // The structured result is in result.fields[0]
        setData(result.fields?.[0] || result);
      })
      .catch((e: any) => setError(String(e)));
  }, [content]);

  // Generate waveform visualization based on protocol
  const waveform = useMemo(() => {
    if (!data) return null;
    const width = 400;
    const height = 100;
    const mid = height / 2;

    // Generate a simple waveform pattern based on frequency
    const points: string[] = [];
    const cycles = Math.min(Math.max(Math.floor(data.frequency / 10_000_000), 2), 20);

    for (let x = 0; x <= width; x++) {
      const t = (x / width) * cycles * Math.PI * 2;
      const y = mid + Math.sin(t) * (height / 3) * (data.protocol === "RAW" ? 0.5 : 1);
      points.push(`${x},${y}`);
    }

    return (
      <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-24 bg-gray-900 rounded">
        {/* Grid lines */}
        <line x1="0" y1={mid} x2={width} y2={mid} stroke="#374151" strokeWidth="0.5" />
        <line x1={width / 2} y1="0" x2={width / 2} y2={height} stroke="#374151" strokeWidth="0.5" />
        {/* Waveform */}
        <polyline
          points={points.join(" ")}
          fill="none"
          stroke="#3b82f6"
          strokeWidth="1.5"
        />
        {/* Labels */}
        <text x="4" y="12" fill="#9ca3af" fontSize="8">{data.protocol}</text>
        <text x={width - 60} y="12" fill="#9ca3af" fontSize="8">{data.frequency_display}</text>
      </svg>
    );
  }, [data]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  return (
    <div className="p-3 bg-gray-800 rounded-lg space-y-3">
      <h4 className="text-sm font-bold text-blue-400">📡 Sub-GHz File</h4>

      {/* Waveform */}
      {waveform}

      {/* Fields */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Frequency</span>
          <div className="text-green-400 font-mono">{data.frequency_display}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Protocol</span>
          <div className="text-yellow-400 font-mono">{data.protocol || "Unknown"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Preset</span>
          <div className="text-gray-300 font-mono text-[10px]">{data.preset}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Bits</span>
          <div className="text-blue-400 font-mono">{data.bit || "—"}</div>
        </div>
        {data.key && (
          <div className="col-span-2 bg-gray-700/50 rounded p-2">
            <span className="text-gray-400">Key</span>
            <div className="text-white font-mono text-[10px] break-all">{data.key}</div>
          </div>
        )}
        {data.is_raw && (
          <div className="col-span-2 bg-orange-900/30 rounded p-2 text-orange-400">
            ⚡ RAW signal detected
          </div>
        )}
      </div>
    </div>
  );
}
