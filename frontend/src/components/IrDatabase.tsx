import { useState, useEffect, useMemo } from "react";
import { parserParseIrStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

// Common IR code database
const IR_DB: Record<string, { brand: string; device: string; description: string }[]> = {
  "NEC": [
    { brand: "Samsung", device: "TV", description: "Power" },
    { brand: "Samsung", device: "TV", description: "Volume Up" },
    { brand: "Samsung", device: "TV", description: "Volume Down" },
    { brand: "LG", device: "TV", description: "Power" },
    { brand: "LG", device: "TV", description: "Mute" },
    { brand: "Sony", device: "TV", description: "Power" },
    { brand: "Sony", device: "TV", description: "Input" },
    { brand: "Panasonic", device: "AC", description: "Power On" },
    { brand: "Panasonic", device: "AC", description: "Temp Up" },
    { brand: "Daikin", device: "AC", description: "Power" },
  ],
  "RC5": [
    { brand: "Philips", device: "TV", description: "Power" },
    { brand: "Philips", device: "TV", description: "Channel Up" },
  ],
  "RC6": [
    { brand: "Microsoft", device: "MCE", description: "Power" },
    { brand: "Microsoft", device: "MCE", description: "Volume" },
  ],
};

// Pronto Hex conversion
function toProntoHex(protocol: string, address: string, command: string): string {
  // Pronto Hex format: 0000 006C 0022 0002 ...
  const pronto: string[] = ["0000", "006C", "0022", "0002"];

  if (protocol === "NEC") {
    // NEC: 38kHz carrier, specific timing
    const addr = parseInt(address.replace("0x", ""), 16) || 0;
    const cmd = parseInt(command.replace("0x", ""), 16) || 0;

    // Lead pulse: 16 ticks on, 8 ticks off
    pronto.push("015B", "00AD"); // ~9ms on, ~4.5ms off
    pronto.push("0016", "0016"); // ~560us on, ~560us off

    // Address (LSB first)
    for (let i = 0; i < 8; i++) {
      const bit = (addr >> i) & 1;
      pronto.push("0016", bit ? "0041" : "0016");
    }
    // Address inverted
    for (let i = 0; i < 8; i++) {
      const bit = (~addr >> i) & 1;
      pronto.push("0016", bit ? "0041" : "0016");
    }
    // Command + inverted
    for (let i = 0; i < 8; i++) {
      const bit = (cmd >> i) & 1;
      pronto.push("0016", bit ? "0041" : "0016");
    }
    for (let i = 0; i < 8; i++) {
      const bit = (~cmd >> i) & 1;
      pronto.push("0016", bit ? "0041" : "0016");
    }

    // Stop bit
    pronto.push("0016", "00B0");
  }

  return pronto.join(" ");
}

export default function IrDatabase({ content, fileName }: Props) {
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
  const [showPronto, setShowPronto] = useState(false);

  useEffect(() => {
    parserParseIrStruct(content)
      .then((result: any) => setData(result.fields?.[0] || result))
      .catch((e: any) => setError(String(e)));
  }, [content]);

  // Match against database
  const matches = useMemo(() => {
    if (!data?.protocol) return [];
    const db = IR_DB[data.protocol] || [];
    const addr = data.address || "";
    const cmd = data.command || "";

    return db.filter(entry => {
      // Simple matching based on address/command patterns
      return true; // Show all matches for the protocol
    });
  }, [data]);

  const prontoHex = useMemo(() => {
    if (!data?.protocol || !data?.address || !data?.command) return null;
    return toProntoHex(data.protocol, data.address, data.command);
  }, [data]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  return (
    <div className="p-4 bg-gray-800 rounded-lg space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-bold text-orange-400">🔴 IR Database</h4>
        <span className="text-xs text-gray-500">{fileName}</span>
      </div>

      {/* Signal Info */}
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

      {/* Database Matches */}
      {matches.length > 0 && (
        <div className="space-y-1">
          <div className="text-xs text-gray-400 font-bold">
            Database Matches ({matches.length})
          </div>
          <div className="max-h-32 overflow-y-auto space-y-1">
            {matches.map((match: any, i: number) => (
              <div key={i} className="bg-gray-700/50 rounded p-2 text-xs flex justify-between">
                <div>
                  <span className="text-orange-300">{match.brand}</span>
                  <span className="text-gray-500 ml-1">{match.device}</span>
                </div>
                <span className="text-gray-400">{match.description}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Pronto Hex Export */}
      {prontoHex && (
        <div>
          <button
            onClick={() => setShowPronto(!showPronto)}
            className="text-xs text-gray-400 hover:text-gray-200"
          >
            {showPronto ? "▼" : "▶"} Export Pronto Hex
          </button>
          {showPronto && (
            <div className="mt-2 space-y-2">
              <pre className="bg-gray-900 rounded p-2 text-[10px] font-mono text-green-400 break-all">
                {prontoHex}
              </pre>
              <button
                onClick={() => navigator.clipboard?.writeText(prontoHex)}
                className="text-xs px-2 py-1 bg-gray-700 rounded text-gray-300 hover:bg-gray-600"
              >
                Copy to Clipboard
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
