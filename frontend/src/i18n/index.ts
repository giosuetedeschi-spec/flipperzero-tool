/**
 * Minimal i18n.
 *
 * A flat key map with `{placeholder}` interpolation and singular/plural
 * variants covers everything this app needs; a library would add a dependency
 * and a runtime for behaviour we can express in a few lines.
 */
import { it } from "./it";
import { en } from "./en";

export type TranslationKey = keyof typeof it;
export type Locale = "it" | "en";

const catalogues: Record<Locale, Record<TranslationKey, string>> = { it, en };

export const SUPPORTED_LOCALES: Locale[] = ["it", "en"];
export const DEFAULT_LOCALE: Locale = "en";

/** Pick a locale from a browser language tag, falling back to English. */
export function resolveLocale(language: string | undefined | null): Locale {
  if (!language) return DEFAULT_LOCALE;
  // Match on the primary subtag so "it-CH" and "it" behave the same.
  const primary = language.toLowerCase().split("-")[0];
  return SUPPORTED_LOCALES.includes(primary as Locale) ? (primary as Locale) : DEFAULT_LOCALE;
}

/**
 * Look up a key, substituting `{name}` placeholders.
 *
 * Falls back to English, then to the key itself. Returning the key rather than
 * an empty string keeps a missing translation visible instead of silently
 * blanking part of the interface.
 */
export function translate(
  locale: Locale,
  key: TranslationKey,
  params?: Record<string, string | number>,
): string {
  const template = catalogues[locale]?.[key] ?? catalogues[DEFAULT_LOCALE][key] ?? key;
  if (!params) return template;

  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/**
 * Choose between the `_one` and `_other` forms of a key.
 *
 * Italian and English both use the singular only for exactly one, so a single
 * rule serves both catalogues.
 */
export function translatePlural(
  locale: Locale,
  baseKey: string,
  count: number,
  params?: Record<string, string | number>,
): string {
  const suffix = count === 1 ? "_one" : "_other";
  return translate(locale, `${baseKey}${suffix}` as TranslationKey, { count, ...params });
}
