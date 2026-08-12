import { useEffect, useState } from "react";

/**
 * Whether the app is on screen.
 *
 * Polling a Flipper while nobody is looking drains two batteries at once, and
 * on BLE it also holds a link that carries only a few kilobytes per second. The
 * Radar pauses when this goes false.
 *
 * `visibilitychange` covers the cases that matter on a phone -- switching apps,
 * locking the screen -- and on desktop covers a hidden tab.
 */
export function useDocumentVisible(): boolean {
  const [visible, setVisible] = useState(() => {
    if (typeof document === "undefined") return true;
    return !document.hidden;
  });

  useEffect(() => {
    if (typeof document === "undefined") return;
    const update = () => setVisible(!document.hidden);
    document.addEventListener("visibilitychange", update);
    return () => document.removeEventListener("visibilitychange", update);
  }, []);

  return visible;
}
