import { useState, useEffect, useMemo } from "react";

interface Props {
  contentA: string;
  contentB: string;
  fileNameA: string;
  fileNameB: string;
}

interface DiffLine {
  type: "same" | "added" | "removed" | "changed";
  lineA: number;
  lineB: number;
  textA: string;
  textB: string;
}

// Simple LCS-based diff
function computeDiff(linesA: string[], linesB: string[]): DiffLine[] {
  const result: DiffLine[] = [];
  const m = linesA.length;
  const n = linesB.length;

  // Build LCS table
  const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (linesA[i - 1] === linesB[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to build diff
  let i = m, j = n;
  const reversed: DiffLine[] = [];

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && linesA[i - 1] === linesB[j - 1]) {
      reversed.push({
        type: "same",
        lineA: i,
        lineB: j,
        textA: linesA[i - 1],
        textB: linesB[j - 1],
      });
      i--; j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      reversed.push({
        type: "added",
        lineA: 0,
        lineB: j,
        textA: "",
        textB: linesB[j - 1],
      });
      j--;
    } else if (i > 0) {
      reversed.push({
        type: "removed",
        lineA: i,
        lineB: 0,
        textA: linesA[i - 1],
        textB: "",
      });
      i--;
    }
  }

  return reversed.reverse();
}

export default function FileDiffViewer({ contentA, contentB, fileNameA, fileNameB }: Props) {
  const [viewMode, setViewMode] = useState<"unified" | "side">("side");

  const diff = useMemo(() => {
    const linesA = contentA.split("\\n");
    const linesB = contentB.split("\\n");
    return computeDiff(linesA, linesB);
  }, [contentA, contentB]);

  const stats = useMemo(() => {
    const added = diff.filter(d => d.type === "added").length;
    const removed = diff.filter(d => d.type === "removed").length;
    const same = diff.filter(d => d.type === "same").length;
    return { added, removed, same, total: diff.length };
  }, [diff]);

  const typeColors: Record<string, string> = {
    same: "text-gray-400",
    added: "text-green-400 bg-green-900/20",
    removed: "text-red-400 bg-red-900/20",
    changed: "text-yellow-400 bg-yellow-900/20",
  };

  const typeIcons: Record<string, string> = {
    same: " ",
    added: "+",
    removed: "-",
    changed: "~",
  };

  return (
    <div className="p-4 bg-gray-800 rounded-lg space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-bold text-cyan-400">📄 File Diff</h4>
        <div className="flex gap-2">
          <button
            onClick={() => setViewMode("side")}
            className={`text-xs px-2 py-1 rounded ${viewMode === "side" ? "bg-cyan-600 text-white" : "bg-gray-700 text-gray-400"}`}
          >
            Side by Side
          </button>
          <button
            onClick={() => setViewMode("unified")}
            className={`text-xs px-2 py-1 rounded ${viewMode === "unified" ? "bg-cyan-600 text-white" : "bg-gray-700 text-gray-400"}`}
          >
            Unified
          </button>
        </div>
      </div>

      {/* File names */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">A:</span>
          <span className="text-red-300 ml-1">{fileNameA}</span>
        </div>
        <div className="bg-gray-700/50 rounded p-2">
          <span className="text-gray-400">B:</span>
          <span className="text-green-300 ml-1">{fileNameB}</span>
        </div>
      </div>

      {/* Stats */}
      <div className="flex gap-3 text-xs">
        <span className="text-gray-400">
          <span className="text-green-400">+{stats.added}</span> added
        </span>
        <span className="text-gray-400">
          <span className="text-red-400">-{stats.removed}</span> removed
        </span>
        <span className="text-gray-400">
          <span className="text-gray-300">{stats.same}</span> unchanged
        </span>
      </div>

      {/* Diff content */}
      <div className="max-h-96 overflow-y-auto font-mono text-[11px]">
        {viewMode === "unified" ? (
          <div className="space-y-px">
            {diff.map((line, i) => (
              <div key={i} className={`flex ${typeColors[line.type]} px-2 py-0.5`}>
                <span className="w-6 text-right mr-2 text-gray-600">
                  {line.lineA || ""}
                </span>
                <span className="w-4 mr-2">{typeIcons[line.type]}</span>
                <span className="flex-1 break-all">{line.textA || line.textB}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-px">
            <div className="space-y-px">
              <div className="text-gray-500 text-center py-1 bg-gray-700/50">{fileNameA}</div>
              {diff.map((line, i) => (
                <div key={i} className={`flex ${typeColors[line.type]} px-2 py-0.5`}>
                  <span className="w-6 text-right mr-2 text-gray-600">
                    {line.lineA || ""}
                  </span>
                  <span className="flex-1 break-all">{line.textA}</span>
                </div>
              ))}
            </div>
            <div className="space-y-px">
              <div className="text-gray-500 text-center py-1 bg-gray-700/50">{fileNameB}</div>
              {diff.map((line, i) => (
                <div key={i} className={`flex ${typeColors[line.type]} px-2 py-0.5`}>
                  <span className="w-6 text-right mr-2 text-gray-600">
                    {line.lineB || ""}
                  </span>
                  <span className="flex-1 break-all">{line.textB}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
