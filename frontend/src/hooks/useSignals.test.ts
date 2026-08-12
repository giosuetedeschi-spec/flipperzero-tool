import { beforeEach, describe, expect, it as test, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { availabilityFor, useSignals } from "./useSignals";
import { signalsCountsByChip, signalsListByChip } from "../services/tauri";

vi.mock("../services/tauri", () => ({
  signalsCountsByChip: vi.fn(async () => []),
  signalsListByChip: vi.fn(async () => []),
}));

const counts = vi.mocked(signalsCountsByChip);
const list = vi.mocked(signalsListByChip);

describe("availabilityFor", () => {
  test("Bluetooth is unavailable over BLE, because the radio is already busy", () => {
    // The Flipper cannot scan for other devices while serving the phone's own
    // BLE link. An empty list would read as "nothing nearby", which is a lie.
    expect(availabilityFor("bluetooth", true, 0)).toBe("unavailable");
  });

  test("Bluetooth is a normal chip over USB", () => {
    expect(availabilityFor("bluetooth", false, 0)).toBe("active");
  });

  test("an add-on module is hidden until something is seen on it", () => {
    // Drawing it regardless would imply the user owns boards they do not have.
    expect(availabilityFor("nrf24", false, 0)).toBe("not_detected");
    expect(availabilityFor("wifi_devboard", false, 0)).toBe("not_detected");
  });

  test("an add-on module appears once it reports something", () => {
    expect(availabilityFor("nrf24", false, 1)).toBe("active");
  });

  test("built-in chips are active even with nothing seen", () => {
    // "Nothing found yet" is a different statement from "no such hardware".
    for (const chip of ["subghz", "nfc", "lfrfid", "ibutton", "infrared", "gpio"] as const) {
      expect(availabilityFor(chip, false, 0), chip).toBe("active");
    }
  });
});

describe("useSignals", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(document, "hidden", { value: false, configurable: true });
  });

  test("asks the backend for nothing while disconnected", () => {
    renderHook(() => useSignals(false, false, false, null));
    expect(counts).not.toHaveBeenCalled();
    expect(list).not.toHaveBeenCalled();
  });

  test("mock mode serves demo data without touching the backend", () => {
    const { result } = renderHook(() => useSignals(false, true, false, null));

    expect(counts).not.toHaveBeenCalled();
    // Two demo Sub-GHz signals, one NFC.
    expect(result.current.signalsByChip.subghz).toHaveLength(2);
    expect(result.current.chips.find((c) => c.chip === "subghz")?.count).toBe(2);
  });

  test("fetches once on connect", async () => {
    renderHook(() => useSignals(true, false, false, null));
    await waitFor(() => expect(counts).toHaveBeenCalled());
  });

  test("does not poll while the app is off screen", () => {
    // Polling unseen drains the phone and the Flipper both.
    Object.defineProperty(document, "hidden", { value: true, configurable: true });
    renderHook(() => useSignals(true, false, false, "subghz"));
    expect(counts).not.toHaveBeenCalled();
  });

  test("reports whether it is actually polling", () => {
    const connected = renderHook(() => useSignals(true, false, false, null));
    expect(connected.result.current.polling).toBe(true);

    const disconnected = renderHook(() => useSignals(false, false, false, null));
    expect(disconnected.result.current.polling).toBe(false);
  });

  test("only the focused chip's signals are fetched", async () => {
    renderHook(() => useSignals(true, false, false, "nfc"));
    await waitFor(() => expect(list).toHaveBeenCalledWith("nfc"));
    // Not one call per chip: that is the whole point of the adaptive schedule.
    expect(list).toHaveBeenCalledTimes(1);
  });

  test("a backend failure surfaces instead of being swallowed", async () => {
    counts.mockRejectedValueOnce(new Error("device went away"));
    const { result } = renderHook(() => useSignals(true, false, false, null));
    await waitFor(() => expect(result.current.error).toBe("device went away"));
  });

  test("mock mode reports no error even if the backend would fail", () => {
    counts.mockRejectedValue(new Error("nothing is connected"));
    const { result } = renderHook(() => useSignals(false, true, false, null));
    expect(result.current.error).toBeNull();
  });

  test("every chip appears in the list, seen or not", () => {
    const { result } = renderHook(() => useSignals(false, false, false, null));
    // The diagram draws from this, so a missing entry is a missing hotspot.
    expect(result.current.chips).toHaveLength(11);
  });
});
