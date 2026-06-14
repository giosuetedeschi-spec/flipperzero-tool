import { useState, useEffect, useCallback } from "react";
import {
  serialListPorts,
  serialConnect,
  serialDisconnect,
  serialIsConnected,
  serialAutodetectConnect,
  type PortInfo,
} from "../services/tauri";

interface Props {
  onConnectionChange?: (connected: boolean) => void;
}

export default function DevicePanel({ onConnectionChange }: Props) {
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [connecting, setConnecting] = useState(false);

  const refreshPorts = useCallback(async () => {
    setScanning(true);
    try {
      const p = await serialListPorts();
      setPorts(p);
      if (p.length > 0 && !selectedPort) {
        setSelectedPort(p[0].port_name);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.toString() : "Failed to list ports");
    } finally {
      setScanning(false);
    }
  }, [selectedPort]);

  useEffect(() => {
    refreshPorts();
    // Check connection status periodically
    const interval = setInterval(async () => {
      try {
        const status = await serialIsConnected();
        setConnected(status);
        onConnectionChange?.(status);
      } catch {
        // ignore
      }
    }, 2000);
    return () => clearInterval(interval);
  }, [refreshPorts, onConnectionChange]);

  const handleConnect = async () => {
    if (!selectedPort) {
      setError("Select a port first");
      return;
    }
    setConnecting(true);
    setError(null);
    try {
      await serialConnect(selectedPort);
      setConnected(true);
      onConnectionChange?.(true);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.toString() : "Connection failed");
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await serialDisconnect();
      setConnected(false);
      onConnectionChange?.(false);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.toString() : "Disconnect failed");
    }
  };

  const handleAutodetect = async () => {
    setConnecting(true);
    setError(null);
    try {
      const found = await serialAutodetectConnect();
      if (found) {
        setConnected(true);
        onConnectionChange?.(true);
        await refreshPorts();
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.toString() : "Flipper not found (VID:PID 0483:5740)");
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="flex flex-col gap-3 p-4 bg-gray-800 rounded-lg">
      <h3 className="text-sm font-bold text-gray-200 uppercase tracking-wider">
        Flipper Device
      </h3>

      {/* Connection status indicator */}
      <div className="flex items-center gap-2">
        <div
          className={`w-3 h-3 rounded-full ${
            connected ? "bg-green-500 animate-pulse" : "bg-red-500"
          }`}
        />
        <span className="text-sm text-gray-300">
          {connected ? "Connected" : "Disconnected"}
        </span>
      </div>

      {/* Port selector */}
      {!connected && (
        <>
          <div className="flex gap-2">
            <select
              className="flex-1 bg-gray-700 text-gray-200 rounded px-3 py-2 text-sm"
              value={selectedPort}
              onChange={(e) => setSelectedPort(e.target.value)}
            >
              <option value="">Select port...</option>
              {ports.map((p) => (
                <option key={p.port_name} value={p.port_name}>
                  {p.port_name} {p.product ? `(${p.product})` : ""}
                </option>
              ))}
            </select>
            <button
              onClick={refreshPorts}
              disabled={scanning}
              className="px-3 py-2 bg-gray-600 hover:bg-gray-500 rounded text-sm text-gray-200 disabled:opacity-50"
              title="Refresh ports"
            >
              {scanning ? "⟳" : "↻"}
            </button>
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleConnect}
              disabled={connecting || !selectedPort}
              className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-sm font-medium text-white disabled:opacity-50"
            >
              {connecting ? "Connecting..." : "Connect"}
            </button>
            <button
              onClick={handleAutodetect}
              disabled={connecting}
              className="px-4 py-2 bg-purple-600 hover:bg-purple-500 rounded text-sm font-medium text-white disabled:opacity-50"
              title="Auto-detect Flipper Zero (VID:PID 0483:5740)"
            >
              Auto
            </button>
          </div>
        </>
      )}

      {connected && (
        <button
          onClick={handleDisconnect}
          className="px-4 py-2 bg-red-600 hover:bg-red-500 rounded text-sm font-medium text-white"
        >
          Disconnect
        </button>
      )}

      {/* Error display */}
      {error && (
        <div className="text-red-400 text-xs bg-red-900/30 rounded p-2">
          {error}
          <button
            onClick={() => setError(null)}
            className="float-right text-red-300 hover:text-red-100"
          >
            ✕
          </button>
        </div>
      )}
    </div>
  );
}
