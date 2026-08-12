import { createContext, useContext } from "react";
import {
  DEFAULT_LOCALE,
  type Locale,
  type TranslationKey,
  resolveLocale,
  translate,
  translatePlural,
} from "./index";

export interface I18n {
  locale: Locale;
  t: (key: TranslationKey, params?: Record<string, string | number>) => string;
  tPlural: (baseKey: string, count: number, params?: Record<string, string | number>) => string;
  setLocale: (locale: Locale) => void;
}

/** Build the translation helpers for a locale. */
export function createI18n(locale: Locale, setLocale: (locale: Locale) => void): I18n {
  return {
    locale,
    t: (key, params) => translate(locale, key, params),
    tPlural: (baseKey, count, params) => translatePlural(locale, baseKey, count, params),
    setLocale,
  };
}

/** The locale to start in: the browser's, when we support it. */
export function detectLocale(): Locale {
  if (typeof navigator === "undefined") return DEFAULT_LOCALE;
  return resolveLocale(navigator.language);
}

export const I18nContext = createContext<I18n>(createI18n(DEFAULT_LOCALE, () => {}));

export function useT(): I18n {
  return useContext(I18nContext);
}
