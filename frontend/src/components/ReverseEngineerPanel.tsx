import { useState, useCallback } from "react";
import { reverseEngineerAnalyze, reverseEngineerAnalyzeFile } from "../services/tauri";

interface AnalysisResult {
  entropy: number;
  total_bytes: number;
  unique_bytes: number;
  patterns: Array<{
    offset: number;
    length: number;
    pattern: number[];
    confidence: number;
    description: string;
  }>;
  matched_protocols: Array<{
    name: string;
    signature: number[];
    offset: number;
    description: string;
  }>;
  inferred_structure: Array<{
    offset: number;
    length: number;
    field_type: string;
    confidence: number;
    value_hex: string;
    value_dec: number | null;
  }>;
  hex_preview: string;
  ascii_preview: string;
}

interface Props {
  currentPath: string;
}

export default function ReverseEngineerPanel({ currentPath }: Props) {
  const [hexInput, setHexInput] = useState("");
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<"hex" | "file">("hex");

  const handleAnalyzeHex = useCallback(async () => {
    if (!hexInput.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await reverseEngineerAnalyze(hexInput.trim());
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [hexInput]);

  const handleAnalyzeFile = useCallback(async (filePath: string) => {
    if (!filePath) return;
    setLoading(true);
    setError(null);
    try {
      const res = await reverseEngineerAnalyzeFile(filePath);
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const getConfidenceColor = (c: number) => {
    if (c >= 0.7) return "text-green-400";
    if (c >= 0.4) return "text-yellow-400";
    return "text-red-400";
  };

  const getFieldTypeColor = (t: string) => {
    switch (t) {
      case "header": return "bg-blue-600";
      case "length": return "bg-purple-600";
      case "payload": return "bg-green-600";
      case "checksum": return "bg-orange-600";
      default: return "bg-gray-600";
    }
  };

  return (
    <div className="w-96 bg-gray-800 border-l border-gray-700 flex flex-col overflow-y-auto">
      <div className="px-3 py-2 border-b border-gray-700 shrink-0">
        <h3 className="text-sm font-bold text-emerald-400">Reverse Engineer</h3>
        <p className="text-xs text-gray-500 mt-1">Analyze raw data to identify protocols and structure</p>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-700 shrink-0">
        <button
          onClick={() => setTab("hex")}
          className={`flex-1 px-3 py-2 text-xs font-medium ${tab === "hex" ? "bg-gray-700 text-emerald-400" : "text-gray-400 hover:text-gray-200"}`}
        >
          Hex Input
        </button>
        <button
          onClick={() => setTab("file")}
          className={`flex-1 px-3 py-2 text-xs font-medium ${tab === "file" ? "bg-gray-700 text-emerald-400" : "text-gray-400 hover:text-gray-200"}`}
        >
          From File
        </button>
      </div>

      <div className="p-3 space-y-3 shrink-0">
        {tab === "hex" ? (
          <>
            <textarea
              value={hexInput}
              onChange={(e) => setHexInput(e.target.value)}
              placeholder="Paste hex data (e.g. AABBCCDDEEFF...)"
              className="w-full h-24 px-2 py-1 bg-gray-900 border border-gray-600 rounded text-xs font-mono text-gray-300 placeholder-gray-600 resize-none"
            />
            <button
              onClick={handleAnalyzeHex}
              disabled={loading || !hexInput.trim()}
              className="w-full px-3 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-600 disabled:text-gray-400 rounded text-sm font-medium"
            >
              {loading ? "Analyzing..." : "Analyze"}
            </button>
          </>
        ) : (
          <>
            <div className="text-xs text-gray-400">
              Analyze a file from the current directory.
            </div>
            <div className="text-xs font-mono text-gray-500 break-all">
              {currentPath || "No directory selected"}
            </div>
            <button
              onClick={() => handleAnalyzeFile(currentPath + "/unknown.bin")}
              disabled={loading || !currentPath}
              className="w-full px-3 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-600 disabled:text-gray-400 rounded text-sm font-medium"
            >
              {loading ? "Analyzing..." : "Analyze Selected File"}
            </button>
          </>
        )}
      </div>

      {error && (
        <div className="mx-3 mb-2 px-2 py-1 bg-red-900/50 border border-red-700 rounded text-xs text-red-300">
          {error}
        </div>
      )}

      {/* Results */}
      {result && (
        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          {/* Overview */}
          <div className="bg-gray-900 rounded-lg p-3 space-y-2">
            <h4 className="text-xs font-bold text-gray-300">Overview</h4>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div>
                <span className="text-gray-500">Size:</span>{" "}
                <span className="text-gray-300">{result.total_bytes} bytes</span>
              </div>
              <div>
                <span className="text-gray-500">Unique bytes:</span>{" "}
                <span className="text-gray-300">{result.unique_bytes}</span>
              </div>
              <div className="col-span-2">
                <span className="text-gray-500">Entropy:</span>{" "}
                <span className={result.entropy > 7 ? "text-red-400" : result.entropy > 5 ? "text-yellow-400" : "text-green-400"}>
                  {result.entropy.toFixed(2)} / 8.0
                </span>
                <span className="text-gray-600 ml-1">
                  ({result.entropy > 7 ? "likely encrypted" : result.entropy > 5 ? "compressed" : "plaintext"})
                </span>
              </div>
            </div>
          </div>

          {/* Hex + ASCII preview */}
          <div className="bg-gray-900 rounded-lg p-3 space-y-1">
            <h4 className="text-xs font-bold text-gray-300">Preview</h4>
            <div className="text-[10px] font-mono text-gray-400 break-all leading-tight">
              {result.hex_preview}
            </div>
            <div className="text-[10px] font-mono text-gray-500 break-all leading-tight">
              {result.ascii_preview}
            </div>
          </div>

          {/* Matched protocols */}
          {result.matched_protocols.length > 0 && (
            <div className="bg-gray-900 rounded-lg p-3 space-y-2">
              <h4 className="text-xs font-bold text-gray-300">
                Matched Protocols ({result.matched_protocols.length})
              </h4>
              {result.matched_protocols.map((p, i) => (
                <div key={i} className="bg-gray-800 rounded p-2 space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-bold text-emerald-400">{p.name}</span>
                  </div>
                  <p className="text-[10px] text-gray-400">{p.description}</p>
                  <div className="text-[10px] font-mono text-gray-500">
                    Signature: {p.signature.map(b => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Patterns */}
          {result.patterns.length > 0 && (
            <div className="bg-gray-900 rounded-lg p-3 space-y-2">
              <h4 className="text-xs font-bold text-gray-300">
                Patterns ({result.patterns.length})
              </h4>
              {result.patterns.slice(0, 10).map((p, i) => (
                <div key={i} className="bg-gray-800 rounded p-2 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] font-mono text-gray-400">
                      Offset {p.offset}, Length {p.length}
                    </span>
                    <span className={`text-[10px] font-bold ${getConfidenceColor(p.confidence)}`}>
                      {(p.confidence * 100).toFixed(0)}%
                    </span>
                  </div>
                  <div className="text-[10px] font-mono text-gray-500 break-all">
                    {p.pattern.map(b => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                  </div>
                  <p className="text-[10px] text-gray-500">{p.description}</p>
                </div>
              ))}
            </div>
          )}

          {/* Inferred structure */}
          {result.inferred_structure.length > 0 && (
            <div className="bg-gray-900 rounded-lg p-3 space-y-2">
              <h4 className="text-xs font-bold text-gray-300">
                Inferred Structure ({result.inferred_structure.length} fields)
              </h4>
              {result.inferred_structure.map((f, i) => (
                <div key={i} className="bg-gray-800 rounded p-2 flex items-center gap-2">
                  <span className={`px-1.5 py-0.5 rounded text-[10px] font-bold text-white ${getFieldTypeColor(f.field_type)}`}>
                    {f.field_type}
                  </span>
                  <span className="text-[10px] font-mono text-gray-400">
                    @{f.offset} [{f.length}]
                  </span>
                  <span className="text-[10px] font-mono text-emerald-400">
                    0x{f.value_hex}
                  </span>
                  {f.value_dec !== null && (
                    <span className="text-[10px] text-gray-500">
                      ({f.value_dec})
                    </span>
                  )}
                  <span className={`text-[10px] ml-auto ${getConfidenceColor(f.confidence)}`}>
                    {(f.confidence * 100).toFixed(0)}%
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
