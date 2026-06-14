import { useState, useEffect } from "react";
import { parserParseNfcStruct } from "../services/tauri";

interface Props {
  content: string;
  fileName: string;
}

// Mifare Classic sector/block helper
function parseMifareBlocks(uid: string, rawContent: string): { sectors: any[], hexDump: string[] } {
  const sectors: any[] = [];
  const hexDump: string[] = [];

  // Generate a basic hex dump from the raw content
  const bytes = new TextEncoder().encode(rawContent);
  for (let i = 0; i < Math.min(bytes.length, 256); i += 16) {
    const hex = Array.from(bytes.slice(i, Math.min(i + 16, bytes.length)))
      .map(b => b.toString(16).padStart(2, '0'))
      .join(' ');
    const ascii = Array.from(bytes.slice(i, Math.min(i + 16, bytes.length)))
      .map(b => b >= 32 && b < 127 ? String.fromCharCode(b) : '.')
      .join('');
    hexDump.push(`${i.toString(16).padStart(4, '0')}: ${hex.padEnd(47)} ${ascii}`);
  }

  // Parse sector info from raw content
  const lines = rawContent.split('\\n');
  let currentSector: any = { blocks: [] };

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('Sector')) {
      if (currentSector.index !== undefined) sectors.push(currentSector);
      const idx = parseInt(trimmed.replace('Sector', '').trim()) || sectors.length;
      currentSector = { index: idx, blocks: [] };
    } else if (trimmed.startsWith('Block')) {
      const parts = trimmed.split(/\\s+/);
      if (parts.length >= 2) {
        const blockIdx = parseInt(parts[0].replace('Block', '').trim()) || currentSector.blocks.length;
        const data = parts.slice(1).join(' ');
        currentSector.blocks.push({ index: blockIdx, data, readable: !data.includes('??') });
      }
    }
  }
  if (currentSector.index !== undefined) sectors.push(currentSector);

  return { sectors, hexDump };
}

// UID analysis
function analyzeUid(uid: string): { manufacturer: string; type: string; bytes: string[] } {
  const cleanUid = uid.replace(/[^0-9A-Fa-f]/g, '');
  const bytes = cleanUid.match(/.{1,2}/g)?.map(b => b.toUpperCase()) || [];

  let manufacturer = "Unknown";
  let type = "Unknown";

  if (bytes.length > 0) {
    switch (bytes[0]) {
      case "04": manufacturer = "NXP (Mifare)"; break;
      case "02": manufacturer = "ST Microelectronics"; break;
      case "08": manufacturer = "NXP (Ultralight)"; break;
      default: manufacturer = `Unknown (0x${bytes[0]})`;
    }
  }

  if (bytes.length === 4) type = "UID4 (Single)";
  else if (bytes.length === 7) type = "UID7 (Double)";
  else if (bytes.length === 10) type = "UID10 (Triple)";

  return { manufacturer, type, bytes };
}

export default function NfcAnalyzerAdvanced({ content, fileName }: Props) {
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
  const [showHex, setShowHex] = useState(false);

  useEffect(() => {
    parserParseNfcStruct(content)
      .then((result: any) => setData(result.fields?.[0] || result))
      .catch((e: any) => setError(String(e)));
  }, [content]);

  if (error) return <div className="text-red-400 text-xs p-2">Parse error: {error}</div>;
  if (!data) return <div className="text-gray-500 text-xs p-2">Loading...</div>;

  const uidInfo = analyzeUid(data.uid || '');
  const { sectors, hexDump } = parseMifareBlocks(data.uid || '', content);

  return (
    <div className="p-4 bg-gray-800 rounded-lg space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-bold text-purple-400">📶 NFC Advanced Analyzer</h4>
        <span className="text-xs text-gray-500">{fileName}</span>
      </div>

      {/* Card Info */}
      <div className={`bg-gradient-to-br ${data.device_type?.includes("Mifare") ? "from-blue-900 to-blue-700" : "from-gray-700 to-gray-600"} rounded-lg p-4`}>
        <div className="text-3xl text-center mb-2">💳</div>
        <div className="text-white font-bold text-center">{data.device_type || "Unknown"}</div>
        <div className="text-white/70 text-xs text-center font-mono mt-1">{data.uid || "No UID"}</div>
      </div>

      {/* UID Analysis */}
      <div className="bg-purple-900/20 border border-purple-800 rounded p-3 space-y-2">
        <div className="text-xs text-purple-400 font-bold">UID Analysis</div>
        <div className="grid grid-cols-2 gap-2 text-xs">
          <div>
            <span className="text-gray-500">Manufacturer:</span>
            <span className="text-purple-300 ml-1">{uidInfo.manufacturer}</span>
          </div>
          <div>
            <span className="text-gray-500">UID Type:</span>
            <span className="text-purple-300 ml-1">{uidInfo.type}</span>
          </div>
          <div className="col-span-2">
            <span className="text-gray-500">Bytes:</span>
            <span className="text-white font-mono ml-1">{uidInfo.bytes.join(' ')}</span>
          </div>
        </div>
      </div>

      {/* ATQA/SAK */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">ATQA</span>
          <div className="text-yellow-400 font-mono">{data.atqa || "—"}</div>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">SAK</span>
          <div className="text-green-400 font-mono">
            {data.sak ? `0x${data.sak.toString(16).padStart(2, '0').toUpperCase()}` : "—"}
          </div>
        </div>
      </div>

      {/* Sectors */}
      {sectors.length > 0 && (
        <div className="space-y-1">
          <span className="text-gray-400 text-xs">Sectors ({sectors.length})</span>
          <div className="max-h-40 overflow-y-auto space-y-1">
            {sectors.map((sector: any, i: number) => (
              <div key={i} className="bg-gray-700/50 rounded p-2 text-xs">
                <div className="flex justify-between items-center mb-1">
                  <span className="text-gray-300 font-bold">Sector {sector.index}</span>
                  <span className="text-gray-500">{sector.blocks?.length || 0} blocks</span>
                </div>
                {sector.blocks?.map((block: any, j: number) => (
                  <div key={j} className="font-mono text-[10px] text-gray-400 flex gap-2">
                    <span className="text-gray-600">B{block.index}</span>
                    <span className={block.readable ? "text-green-400" : "text-red-400"}>
                      {block.data}
                    </span>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Hex Dump Toggle */}
      <div>
        <button
          onClick={() => setShowHex(!showHex)}
          className="text-xs text-gray-400 hover:text-gray-200"
        >
          {showHex ? "▼" : "▶"} Hex Dump
        </button>
        {showHex && (
          <pre className="mt-2 bg-gray-900 rounded p-2 text-[10px] font-mono text-gray-400 max-h-40 overflow-y-auto">
            {hexDump.join('\\n') || "No data"}
          </pre>
        )}
      </div>
    </div>
  );
}
