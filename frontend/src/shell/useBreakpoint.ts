import { useEffect, useState } from "react";

/**
 * Whether the app is running in a phone-shaped viewport.
 *
 * Drives layout only, never capability: a narrow desktop window gets the mobile
 * layout, and that is correct. What the platform can actually *do* -- USB
 * serial, shelling out to uFBT -- is decided in Rust with `#[cfg(desktop)]`, not
 * from the viewport width.
 */

/** Tailwind's `md`. Below this a two-pane layout stops being usable. */
const MOBILE_MAX_WIDTH = 768;

export function useIsMobileLayout(): boolean {
  const [isMobile, setIsMobile] = useState(() => {
    if (typeof window === "undefined" || !window.matchMedia) return false;
    return window.matchMedia(`(max-width: ${MOBILE_MAX_WIDTH - 1}px)`).matches;
  });

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const query = window.matchMedia(`(max-width: ${MOBILE_MAX_WIDTH - 1}px)`);
    const update = (event: MediaQueryListEvent) => setIsMobile(event.matches);
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, []);

  return isMobile;
}
