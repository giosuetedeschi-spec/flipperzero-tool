import { useCallback, useEffect, useRef, useState } from "react";
import type { ChipSlug, Signal } from "../types/signals";
import { isExternalModule } from "../types/signals";
import { signalsCountsByChip, signalsListByChip } from "../services/tauri";
import type { ChipAvailability, ChipStatus } from "../components/radar/FlipperSchematic";
import { DEMO_SIGNALS } from "../mock/demoSignals";

/**
 * Feeds the Radar.
 *
 * Polling is adaptive on purpose. The chip the user is looking at refreshes
 * quickly; the rest refresh slowly. Querying every radio at full rate would
 * drain the Flipper's battery and saturate a BLE link that carries only a few
 * kilobytes per second, buying a liveness the user cannot even see while making
 * file transfers crawl.
 */

/** How often the chip currently on screen refreshes. */
const FOREGROUND_INTERVAL_MS = 1_500;
/** How often the per-chip counts refresh. */
const BACKGROUND_INTERVAL_MS = 15_000;

export interface SignalsState {
  chips: ChipStatus[];
  signalsByChip: Record<string, Signal[]>;
  error: string | null;
  refresh: () => void;
}

const BUILT_IN_CHIPS: ChipSlug[] = [
  "subghz",
  "nfc",
  "lfrfid",
  "ibutton",
  "infrared",
  "bluetooth",
  "gpio",
];

const EXTERNAL_CHIPS: ChipSlug[] = ["wifi_devboard", "external_cc1101", "nrf24", "unknown_module"];

/** What the Radar should say about a chip on the current link. */
export function availabilityFor(
  chip: ChipSlug,
  transportIsBle: boolean,
  count: number,
): ChipAvailability {
  // The Flipper's BLE radio is busy serving the phone, so it cannot also scan
  // for surrounding Bluetooth devices. Saying so beats an empty list, which
  // would read as "nothing nearby".
  if (chip === "bluetooth" && transportIsBle) return "unavailable";
  // Add-on boards only appear once something has actually been seen on them.
  if (count === 0 && isExternalModule(chip)) return "not_detected";
  return "active";
}

export function useSignals(
  connected: boolean,
  mockMode: boolean,
  transportIsBle: boolean,
  focusedChip: ChipSlug | null,
): SignalsState {
  const [counts, setCounts] = useState<Record<string, number>>({});
  const [signalsByChip, setSignalsByChip] = useState<Record<string, Signal[]>>({});
  const [error, setError] = useState<string | null>(null);

  // Held in a ref so changing which chip is open does not tear down and rebuild
  // the polling timers on every tap.
  const focusedRef = useRef<ChipSlug | null>(focusedChip);
  focusedRef.current = focusedChip;

  const loadDemo = useCallback(() => {
    const grouped: Record<string, Signal[]> = {};
    for (const signal of DEMO_SIGNALS) {
      (grouped[signal.chip] ??= []).push(signal);
    }
    const demoCounts = Object.fromEntries(
      Object.entries(grouped).map(([chip, list]) => [chip, list.length]),
    );
    setSignalsByChip(grouped);
    setCounts(demoCounts);
    setError(null);
  }, []);

  const refreshCounts = useCallback(async () => {
    try {
      setCounts(Object.fromEntries(await signalsCountsByChip()));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const refreshFocused = useCallback(async () => {
    const chip = focusedRef.current;
    if (!chip) return;
    try {
      const signals = await signalsListByChip(chip);
      setSignalsByChip((previous) => ({ ...previous, [chip]: signals }));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const refresh = useCallback(() => {
    if (mockMode) {
      loadDemo();
      return;
    }
    void refreshCounts();
    void refreshFocused();
  }, [mockMode, loadDemo, refreshCounts, refreshFocused]);

  useEffect(() => {
    if (mockMode) {
      loadDemo();
      return;
    }
    if (!connected) return;

    void refreshCounts();
    void refreshFocused();
    const fast = setInterval(() => void refreshFocused(), FOREGROUND_INTERVAL_MS);
    const slow = setInterval(() => void refreshCounts(), BACKGROUND_INTERVAL_MS);
    return () => {
      clearInterval(fast);
      clearInterval(slow);
    };
  }, [connected, mockMode, loadDemo, refreshCounts, refreshFocused]);

  const chips: ChipStatus[] = [...BUILT_IN_CHIPS, ...EXTERNAL_CHIPS].map((chip) => {
    const count = counts[chip] ?? 0;
    return { chip, count, availability: availabilityFor(chip, transportIsBle, count) };
  });

  return { chips, signalsByChip, error, refresh };
}
