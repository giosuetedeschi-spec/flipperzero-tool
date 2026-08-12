import { beforeEach, describe, expect, it as test, vi, afterEach } from "vitest";
import { act, renderHook } from "@testing-library/react";
import { useDocumentVisible } from "./useDocumentVisible";
import { isGeotaggingEnabled, setGeotaggingEnabled, useGeolocation } from "./useGeolocation";

/** Drive `document.hidden`, which jsdom exposes but never changes on its own. */
function setHidden(hidden: boolean) {
  Object.defineProperty(document, "hidden", { value: hidden, configurable: true });
  document.dispatchEvent(new Event("visibilitychange"));
}

describe("useDocumentVisible", () => {
  afterEach(() => setHidden(false));

  test("starts from the document's current state", () => {
    setHidden(false);
    const { result } = renderHook(() => useDocumentVisible());
    expect(result.current).toBe(true);
  });

  test("follows the app off screen and back", () => {
    const { result } = renderHook(() => useDocumentVisible());

    act(() => setHidden(true));
    expect(result.current).toBe(false);

    act(() => setHidden(false));
    expect(result.current).toBe(true);
  });

  test("stops listening once unmounted", () => {
    // A leaked listener would keep a torn-down Radar polling forever.
    const remove = vi.spyOn(document, "removeEventListener");
    const { unmount } = renderHook(() => useDocumentVisible());
    unmount();
    expect(remove).toHaveBeenCalledWith("visibilitychange", expect.any(Function));
  });
});

describe("useGeolocation", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.unstubAllGlobals();
  });

  test("is off until the user turns it on", () => {
    // Location is the most sensitive thing this app could record, so nothing is
    // requested by default.
    const watchPosition = vi.fn();
    vi.stubGlobal("navigator", { geolocation: { watchPosition, clearWatch: vi.fn() } });

    const { result } = renderHook(() => useGeolocation());

    expect(result.current.enabled).toBe(false);
    expect(result.current.unavailableReason).toBe("disabled");
    expect(watchPosition).not.toHaveBeenCalled();
  });

  test("starts watching once enabled, and reports a fix", () => {
    const watchPosition = vi.fn((success: PositionCallback) => {
      success({
        coords: { latitude: 45.0703, longitude: 7.6869 },
      } as GeolocationPosition);
      return 1;
    });
    vi.stubGlobal("navigator", { geolocation: { watchPosition, clearWatch: vi.fn() } });

    const { result } = renderHook(() => useGeolocation());
    act(() => result.current.setEnabled(true));

    expect(watchPosition).toHaveBeenCalled();
    expect(result.current.position).toEqual({ latitude: 45.0703, longitude: 7.6869 });
    expect(result.current.unavailableReason).toBeNull();
  });

  test("turning it off drops the last known position", () => {
    // Merely stopping updates would keep tagging sightings with a stale fix
    // after the user asked it to stop.
    const clearWatch = vi.fn();
    vi.stubGlobal("navigator", {
      geolocation: {
        watchPosition: (success: PositionCallback) => {
          success({ coords: { latitude: 1, longitude: 2 } } as GeolocationPosition);
          return 7;
        },
        clearWatch,
      },
    });

    const { result } = renderHook(() => useGeolocation());
    act(() => result.current.setEnabled(true));
    expect(result.current.position).not.toBeNull();

    act(() => result.current.setEnabled(false));
    expect(result.current.position).toBeNull();
    expect(clearWatch).toHaveBeenCalledWith(7);
  });

  test("distinguishes a refusal from simply having no fix", () => {
    // One is permanent until the user changes a setting; the other clears by
    // stepping outside. The UI says different things about them.
    const denied = { code: 1, PERMISSION_DENIED: 1 } as GeolocationPositionError;
    vi.stubGlobal("navigator", {
      geolocation: {
        watchPosition: (_ok: PositionCallback, fail: PositionErrorCallback) => {
          fail(denied);
          return 1;
        },
        clearWatch: vi.fn(),
      },
    });

    const { result } = renderHook(() => useGeolocation());
    act(() => result.current.setEnabled(true));
    expect(result.current.unavailableReason).toBe("denied");
  });

  test("a timeout indoors is reported as no fix, not a refusal", () => {
    const timedOut = { code: 3, PERMISSION_DENIED: 1 } as GeolocationPositionError;
    vi.stubGlobal("navigator", {
      geolocation: {
        watchPosition: (_ok: PositionCallback, fail: PositionErrorCallback) => {
          fail(timedOut);
          return 1;
        },
        clearWatch: vi.fn(),
      },
    });

    const { result } = renderHook(() => useGeolocation());
    act(() => result.current.setEnabled(true));
    expect(result.current.unavailableReason).toBe("no_fix");
  });

  test("a platform without geolocation says so instead of failing", () => {
    vi.stubGlobal("navigator", {});

    const { result } = renderHook(() => useGeolocation());
    act(() => result.current.setEnabled(true));
    expect(result.current.unavailableReason).toBe("unsupported");
    expect(result.current.position).toBeNull();
  });

  test("the choice survives a restart", () => {
    expect(isGeotaggingEnabled()).toBe(false);
    setGeotaggingEnabled(true);
    expect(isGeotaggingEnabled()).toBe(true);
    setGeotaggingEnabled(false);
    expect(isGeotaggingEnabled()).toBe(false);
  });
});
