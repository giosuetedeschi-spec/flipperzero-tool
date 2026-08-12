/**
 * Map RSSI to a 0-100 bar.
 *
 * -100 dBm is roughly the noise floor and -30 dBm is very close, so the useful
 * range is clamped to that rather than to the full theoretical span, which would
 * leave every real reading bunched at one end of the bar.
 *
 * Lives apart from the component that uses it so the component file exports
 * nothing but a component, which is what React Fast Refresh requires.
 */
export function rssiToPercent(rssiDbm: number): number {
  const clamped = Math.min(-30, Math.max(-100, rssiDbm));
  return Math.round(((clamped + 100) / 70) * 100);
}
