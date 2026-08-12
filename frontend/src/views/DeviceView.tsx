import { useCallback, useEffect, useState } from "react";
import SerialPanel from "../components/SerialPanel";
import {
  getErrorMessage,
  serialConnect,
  serialDisconnect,
  serialListPorts,
  type PortInfo,
} from "../services/tauri";
import { useT } from "../i18n/useT";

/**
 * Connection management: pick a port, connect, see what is attached.
 *
 * SerialPanel was written and never rendered; this supplies the state it always
 * expected. On mobile the port list comes back empty with an error, which is the
 * honest answer -- `serialport` is not compiled for iOS or Android, and the BLE
 * transport that replaces it is not wired into the UI yet.
 */

interface Props {
  connected: boolean;
  onConnectionChange: (connected: boolean) => void;
}

export default function DeviceView({ connected, onConnectionChange }: Props) {
  const { t } = useT();
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const found = await serialListPorts();
      setPorts(found);
      setError(null);
      // Preselect the only candidate; with one device attached, making the user
      // pick from a list of one is pure friction.
      if (found.length === 1) setSelectedPort(found[0].port_name);
    } catch (err) {
      setPorts([]);
      setError(getErrorMessage(err));
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- standard fetch-on-mount pattern
    void refresh();
  }, [refresh]);

  const handleConnect = async () => {
    try {
      await serialConnect(selectedPort);
      onConnectionChange(true);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err));
      onConnectionChange(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await serialDisconnect();
    } catch {
      /* already gone; the UI state below is what matters */
    }
    onConnectionChange(false);
  };

  return (
    <div className="flex flex-col gap-4 overflow-auto p-4" data-testid="device-view">
      <header>
        <h1 className="text-xl font-bold text-white">{t("nav.device")}</h1>
        <p className="text-sm text-gray-400">{t("device.subtitle")}</p>
      </header>

      <SerialPanel
        ports={ports}
        selectedPort={selectedPort}
        connected={connected}
        error={error}
        onSelectPort={setSelectedPort}
        onRefresh={() => void refresh()}
        onConnect={() => void handleConnect()}
        onDisconnect={() => void handleDisconnect()}
        onCloseError={() => setError(null)}
      />
    </div>
  );
}
