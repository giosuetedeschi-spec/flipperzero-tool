import { useCallback, useEffect, useRef, useState } from "react";
import type { GeoPoint } from "../types/signals";

/**
 * Where the phone is, for tagging signal sightings.
 *
 * Strictly opt-in. Location is the most sensitive thing this app could record --
 * a geotagged history of every badge and gate remote near you is a map of where
 * you go -- so nothing is requested until the user turns it on, and turning it
 * off drops the last known position rather than merely stopping updates.
 *
 * A missing fix is a normal state, not an error. Indoors, which is where NFC
 * and RFID reads happen, GPS routinely fails; a sighting simply records without
 * a position rather than failing or nagging.
 */

const STORAGE_KEY = "flipper_geotagging_enabled";

/** Stale enough to be misleading rather than helpful. */
const MAX_POSITION_AGE_MS = 60_000;

export function isGeotaggingEnabled(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(STORAGE_KEY) === "1";
}

export function setGeotaggingEnabled(enabled: boolean): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, enabled ? "1" : "0");
}

type GeolocationReason = "disabled" | "denied" | "unsupported" | "no_fix" | null;

export interface GeolocationState {
  enabled: boolean;
  setEnabled: (enabled: boolean) => void;
  /** Last known position, or null when unavailable or switched off. */
  position: GeoPoint | null;
  /** Why there is no position, for the UI to explain rather than hide. */
  unavailableReason: GeolocationReason;
}

export function useGeolocation(): GeolocationState {
  const [enabled, setEnabledState] = useState(isGeotaggingEnabled);
  const [position, setPosition] = useState<GeoPoint | null>(null);
  /** Only ever set from the geolocation callbacks, which are asynchronous. */
  const [watchFailure, setWatchFailure] = useState<"denied" | "no_fix" | null>(null);

  // Whether the platform offers geolocation at all is a fixed fact about the
  // environment, so it is derived at render rather than written into state from
  // an effect -- which would be a synchronous setState in an effect.
  const supported = typeof navigator !== "undefined" && !!navigator.geolocation;

  const watchIdRef = useRef<number | null>(null);

  const setEnabled = useCallback((next: boolean) => {
    setGeotaggingEnabled(next);
    setEnabledState(next);
    if (!next) {
      // Drop the fix rather than just stopping updates: leaving it behind would
      // keep tagging sightings after the user said to stop.
      setPosition(null);
      setWatchFailure(null);
    }
  }, []);

  useEffect(() => {
    if (!enabled || !supported) return;

    const id = navigator.geolocation.watchPosition(
      (fix) => {
        setPosition({ latitude: fix.coords.latitude, longitude: fix.coords.longitude });
        setWatchFailure(null);
      },
      (error) => {
        // Denial is permanent until the user changes it; no fix is temporary.
        // The UI says different things about them, so they stay distinct.
        setPosition(null);
        setWatchFailure(error.code === error.PERMISSION_DENIED ? "denied" : "no_fix");
      },
      { enableHighAccuracy: false, maximumAge: MAX_POSITION_AGE_MS, timeout: 15_000 },
    );
    watchIdRef.current = id;

    return () => {
      if (watchIdRef.current !== null) {
        navigator.geolocation.clearWatch(watchIdRef.current);
        watchIdRef.current = null;
      }
    };
  }, [enabled, supported]);

  // Derived in priority order: switched off, then unsupported, then whatever
  // the watch last reported.
  const unavailableReason: GeolocationReason = !enabled
    ? "disabled"
    : !supported
      ? "unsupported"
      : watchFailure;

  return { enabled, setEnabled, position, unavailableReason };
}
