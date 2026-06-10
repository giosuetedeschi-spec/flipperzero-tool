import { type PortInfo } from "../services/tauri";

interface Props {
  ports: PortInfo[];
  selectedPort: string;
  connected: boolean;
  error: string | null;
  onSelectPort: (port: string) => void;
  onRefresh: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onCloseError: () => void;
}

export default function SerialPanel({
  ports, selectedPort, connected, error,
  onSelectPort, onRefresh, onConnect, onDisconnect, onCloseError,
}: Props) {
  return (
    <>
      <div className="flex items-center gap-2">
        <select
          value={selectedPort}
          onChange={(e) => onSelectPort(e.target.value)}
          className="px-2 py-1 bg-gray-700 rounded text-sm min-w-48 disabled:opacity-50"
          disabled={connected}
        >
          <option value="">-- Select Port --</option>
          {ports.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name} ({p.description || p.port_type})
            </option>
          ))}
        </select>
        {!connected ? (
          <>
            <button
              onClick={onRefresh}
              className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
              title="Refresh ports"
            >
              🔄
            </button>
            <button
              onClick={onConnect}
              disabled={!selectedPort}
              className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 rounded text-sm font-medium"
            >
              Connect
            </button>
          </>
        ) : (
          <button
            onClick={onDisconnect}
            className="px-3 py-1 bg-red-600 hover:bg-red-500 rounded text-sm font-medium"
          >
            Disconnect
          </button>
        )}
        {connected && (
          <span className="text-xs text-emerald-400 flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            Connected
          </span>
        )}
      </div>
      {error && (
        <div className="bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center justify-between">
          <span className="text-red-200 text-sm">⚠️ Serial: {error}</span>
          <button onClick={onCloseError} className="text-red-300 hover:text-red-100 text-sm">✕</button>
        </div>
      )}
    </>
  );
}
