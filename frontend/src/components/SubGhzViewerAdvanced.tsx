import { useState, useEffect, useRef, useCallback } from "react";
import { parserParseSubStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

interface ProtocolInfo {
  name: string;
  frequency: string;
  modulation: string;
}

// Common SubGHz protocol database
const PROTOCOL_DB: Record<string, ProtocolInfo> = {
  "Princeton": { name: "Princeton", frequency: "315/433 MHz", modulation: "OOK" },
  "PWM": { name: "PWM", frequency: "315/433 MHz", modulation: "PWM" },
  "Manchester": { name: "Manchester", frequency: "433 MHz", modulation: "OOK" },
  "RAW": { name: "RAW", frequency: "Various", modulation: "Raw" },
  "NEC": { name: "NEC (IR)", frequency: "38 kHz", modulation: "PPM" },
};

export default function SubGhzViewerAdvanced({ content, fileName }: Props) {
  const [data, setData] = useState<any>(null);
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    parserParseSubStruct(content)
      .then((result: any) => setData(result.fields?.[0] || result))
      .catch((e: any) => setError(String(e)));
  }, [content]);

  // Auto-detect protocol
  const detectedProtocol = data?.protocol
    ? PROTOCOL_DB[data.protocol] || { name: data.protocol, frequency: "Unknown", modulation: "Unknown" }
    : null;

  // Generate detailed waveform
  const generateWaveform = useCallback(() => {
    if (!data) return "";
    const width = 600 * zoom;
    const height = 120;
    const mid = height / 2;
    const points: string[] = [];
    const dataPoints: string[] = [];

    // Different patterns for different protocols
    const pattern = data.protocol || "Unknown";
    const freq = data.frequency || 433920000;

    for (let x = 0; x <= width; x++) {
      const cyclePos = (x / width) * 20; // 20 full cycles
      let y = mid;

      if (pattern.includes("RAW")) {
        // Random-ish pattern for RAW
        y = mid + (Math.sin(cyclePos * Math.PI * 2) * 0.3 + Math.sin(cyclePos * Math.PI * 7) * 0.2) * (height / 3);
      } else if (pattern === "Princeton") {
        // Princeton: short pulse = 0, long pulse = 1
        const bitPos = Math.floor(cyclePos / 4) % 2;
        const subPos = (cyclePos / 4) % 1;
        y = bitPos === 0
          ? (subPos < 0.3 ? mid - height / 3 : mid)
          : (subPos < 0.6 ? mid - height / 3 : mid);
      } else if (pattern === "Manchester") {
        // Manchester encoding
        const bitPos = Math.floor(cyclePos / 2) % 2;
        const subPos = (cyclePos / 2) % 1;
        y = bitPos === 0
          ? (subPos < 0.5 ? mid - height / 3 : mid + height / 3)
          : (subPos < 0.5 ? mid + height / 3 : mid - height / 3);
      } else {
        // Default sine
        y = mid + Math.sin(cyclePos * Math.PI * 2) * (height / 3);
      }

      points.push(`${(x + offset) * zoom},${y}`);
      dataPoints.push(`${x},${y}`);
    }

    return dataPoints.join(" ");
  }, [data, zoom, offset]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  return (
    <div className="p-4 bg-gray-800 rounded-lg space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-bold text-blue-400">📡 SubGHz Analyzer</h4>
        <span className="text-xs text-gray-500">{fileName}</span>
      </div>

      {/* Interactive Waveform */}
      <div className="space-y-2">
        <div className="flex items-center gap-3 text-xs">
          <span className="text-gray-400">Zoom:</span>
          <input
            type="range"
            min="0.5"
            max="5"
            step="0.5"
            value={zoom}
            onChange={(e) => setZoom(Number(e.target.value))}
            className="flex-1"
          />
          <span className="text-blue-400 w-8">{zoom}x</span>
          <button
            onClick={() => setOffset(o => Math.max(0, o - 50))}
            className="px-2 py-1 bg-gray-700 rounded text-gray-300 hover:bg-gray-600"
          >←</button>
          <button
            onClick={() => setOffset(o => o + 50)}
            className="px-2 py-1 bg-gray-700 rounded text-gray-300 hover:bg-gray-600"
          >→</button>
        </div>

        <svg
          ref={svgRef}
          viewBox={`0 0 ${600 * zoom} 120`}
          className="w-full h-32 bg-gray-900 rounded border border-gray-700 cursor-crosshair"
          preserveAspectRatio="xMidYMid meet"
        >
          {/* Grid */}
          <defs>
            <pattern id="grid" width="50" height="20" patternUnits="userSpaceOnUse">
              <path d="M 50 0 L 0 0 0 20" fill="none" stroke="#1f2937" strokeWidth="0.5" />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
          <line x1="0" y1="60" x2={600 * zoom} y2="60" stroke="#374151" strokeWidth="1" />

          {/* Waveform */}
          <polyline
            points={generateWaveform()}
            fill="none"
            stroke="#3b82f6"
            strokeWidth="1.5"
          />

          {/* Labels */}
          <text x="8" y="14" fill="#6b7280" fontSize="9">{data.protocol || "Unknown"}</text>
          <text x="8" y="26" fill="#6b7280" fontSize="8">{data.frequency_display || data.frequency}</text>
        </svg>
      </div>

      {/* Protocol Detection */}
      {detectedProtocol && (
        <div className="bg-blue-900/20 border border-blue-800 rounded p-2">
          <div className="text-xs text-blue-400 font-bold mb-1">Auto-detected Protocol</div>
          <div className="grid grid-cols-3 gap-2 text-xs">
            <div>
              <span className="text-gray-500">Name:</span>
              <span className="text-blue-300 ml-1">{detectedProtocol.name}</span>
            </div>
            <div>
              <span className="text-gray-500">Freq:</span>
              <span className="text-green-300 ml-1">{detectedProtocol.frequency}</span>
            </div>
            <div>
              <span className="text-gray-500">Mod:</span>
              <span className="text-yellow-300 ml-1">{detectedProtocol.modulation}</span>
            </div>
          </div>
        </div>
      )}

      {/* Signal Details */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Frequency</span>
          <div className="text-green-400 font-mono">{data.frequency_display || `${data.frequency} Hz`}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">Bit Length</span>
          <div className="text-blue-400 font-mono">{data.bit || "—"} bits</div>
        </div>
        {data.key && (
          <div className="col-span-2 bg-gray-700/50 rounded p-2">
            <span className="text-gray-400">Key</span>
            <div className="text-white font-mono text-[10px] break-all">{data.key}</div>
          </div>
        )}
      </div>
    </div>
  );
}
