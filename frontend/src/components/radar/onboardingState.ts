/**
 * Whether the first-run Radar tour has already been shown.
 *
 * Lives apart from the component so that file exports only a component, which
 * is what React Fast Refresh requires.
 */
const STORAGE_KEY = "flipper_radar_onboarding_seen";

export function hasSeenOnboarding(): boolean {
  if (typeof localStorage === "undefined") return true;
  return localStorage.getItem(STORAGE_KEY) === "1";
}

export function markOnboardingSeen(): void {
  if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, "1");
}
