import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ChipSlug, Signal } from "../types/signals";
import { isExternalModule } from "../types/signals";
import { signalsCountsByChip, signalsListByChip } from "../services/tauri";
import type { ChipAvailability, ChipStatus } from "../components/radar/FlipperSchematic";
import { DEMO_SIGNALS } from "../mock/demoSignals";
import { useDocumentVisible } from "./useDocumentVisible";

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
  /** False while polling is paused because the app is off screen. */
  polling: boolean;
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
  const visible = useDocumentVisible();

  // Held in a ref so changing which chip is open does not tear down and rebuild
  // the polling timers on every tap. Synced in an effect rather than during
  // render, since a render may be discarded and must stay side-effect free.
  const focusedRef = useRef<ChipSlug | null>(focusedChip);
  useEffect(() => {
    focusedRef.current = focusedChip;
  }, [focusedChip]);

  // Mock data is derived, never stored. Pushing it through state would mean
  // writing state from inside an effect, and the demo set is a constant -- there
  // is nothing to keep in sync.
  const demo = useMemo(() => {
    const signalsByChip: Record<string, Signal[]> = {};
    for (const signal of DEMO_SIGNALS) {
      (signalsByChip[signal.chip] ??= []).push(signal);
    }
    const counts = Object.fromEntries(
      Object.entries(signalsByChip).map(([chip, list]) => [chip, list.length]),
    );
    return { signalsByChip, counts };
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
    // Nothing to refresh in mock mode: the demo set is derived, not fetched.
    if (mockMode) return;
    void refreshCounts();
    void refreshFocused();
  }, [mockMode, refreshCounts, refreshFocused]);

  useEffect(() => {
    // Nobody is looking, so nothing needs refreshing. Leaving the timers running
    // would drain the phone and the Flipper for data no one can see.
    if (mockMode || !connected || !visible) return;

    // The lint rule traces setState through these async callbacks, but the
    // writes happen after an await, not synchronously in the effect. Fetching
    // once on connect is the point of the effect; without it the Radar would
    // stay blank until the first interval fires.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- standard fetch-on-mount pattern
    void refreshCounts();
    void refreshFocused();
    const fast = setInterval(() => void refreshFocused(), FOREGROUND_INTERVAL_MS);
    const slow = setInterval(() => void refreshCounts(), BACKGROUND_INTERVAL_MS);
    return () => {
      clearInterval(fast);
      clearInterval(slow);
    };
  }, [connected, mockMode, visible, refreshCounts, refreshFocused]);

  const effectiveCounts = mockMode ? demo.counts : counts;
  const effectiveSignals = mockMode ? demo.signalsByChip : signalsByChip;

  const chips: ChipStatus[] = [...BUILT_IN_CHIPS, ...EXTERNAL_CHIPS].map((chip) => {
    const count = effectiveCounts[chip] ?? 0;
    return { chip, count, availability: availabilityFor(chip, transportIsBle, count) };
  });

  return {
    chips,
    signalsByChip: effectiveSignals,
    error: mockMode ? null : error,
    refresh,
    polling: connected && !mockMode && visible,
  };
}
