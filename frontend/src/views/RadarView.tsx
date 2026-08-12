import { useState } from "react";
import type { ActionId, ChipSlug, Signal } from "../types/signals";
import { requiresTransmission } from "../types/signals";
import FlipperSchematic from "../components/radar/FlipperSchematic";
import ChipSheet from "../components/radar/ChipSheet";
import TransmitGate from "../components/radar/TransmitGate";
import { useSignals } from "../hooks/useSignals";
import { useT } from "../i18n/useT";

/**
 * The Radar screen: the diagram, the drill-down sheet, and the transmit gate.
 *
 * Owns which chip is open and whether a transmission is waiting on the user.
 * The presentational pieces below stay free of that state so they remain
 * testable in isolation.
 */

interface Props {
  connected: boolean;
  mockMode: boolean;
  /** BLE occupies the Flipper's own radio, which changes what it can scan. */
  transportIsBle?: boolean;
  onPerform?: (signal: Signal, action: ActionId) => void;
}

export default function RadarView({
  connected,
  mockMode,
  transportIsBle = false,
  onPerform,
}: Props) {
  const { t } = useT();
  const [selectedChip, setSelectedChip] = useState<ChipSlug | null>(null);
  const [pending, setPending] = useState<{ signal: Signal; action: ActionId } | null>(null);

  const { chips, signalsByChip, error } = useSignals(
    connected,
    mockMode,
    transportIsBle,
    selectedChip,
  );

  const handleAction = (signal: Signal, action: ActionId) => {
    // Anything that makes the Flipper transmit stops here until the user has
    // read the warning and said yes.
    if (requiresTransmission(action)) {
      setPending({ signal, action });
      return;
    }
    onPerform?.(signal, action);
  };

  const confirmPending = () => {
    if (pending) onPerform?.(pending.signal, pending.action);
    setPending(null);
  };

  const selectedStatus = chips.find((c) => c.chip === selectedChip);

  return (
    <div className="flex flex-col gap-4 p-4" data-testid="radar-view">
      <header>
        <h1 className="text-xl font-bold text-white">{t("radar.title")}</h1>
        <p className="text-sm text-gray-400">{t("radar.subtitle")}</p>
      </header>

      {!connected && !mockMode && (
        <div
          data-testid="radar-disconnected"
          className="rounded-xl border border-gray-700 bg-gray-800 p-4 text-center"
        >
          <p className="text-sm font-medium text-gray-200">{t("radar.disconnected")}</p>
          <p className="mt-1 text-xs text-gray-500">{t("radar.disconnected.hint")}</p>
        </div>
      )}

      {error && (
        <p role="alert" className="rounded-lg bg-red-950 p-3 text-sm text-red-200">
          {error}
        </p>
      )}

      <FlipperSchematic chips={chips} selected={selectedChip} onSelect={setSelectedChip} />

      {selectedChip && selectedStatus && (
        <ChipSheet
          chip={selectedChip}
          signals={signalsByChip[selectedChip] ?? []}
          availability={selectedStatus.availability}
          onClose={() => setSelectedChip(null)}
          onAction={handleAction}
        />
      )}

      {pending && (
        <TransmitGate
          action={pending.action}
          onConfirm={confirmPending}
          onCancel={() => setPending(null)}
        />
      )}
    </div>
  );
}
