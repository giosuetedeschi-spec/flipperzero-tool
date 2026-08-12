/**
 * The app's top-level sections.
 *
 * Kept apart from the shell component so that file exports only a component,
 * which is what React Fast Refresh requires.
 */
export const SECTIONS = ["radar", "files", "tools", "device", "settings"] as const;

export type Section = (typeof SECTIONS)[number];
