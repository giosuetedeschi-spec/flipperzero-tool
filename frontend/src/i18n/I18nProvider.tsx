import { useMemo, useState, type ReactNode } from "react";
import { I18nContext, createI18n, detectLocale } from "./useT";
import type { Locale } from "./index";

/**
 * Supplies the translation helpers to the tree.
 *
 * Starts from the browser's language and persists an explicit choice, so a user
 * who switches to English does not get Italian back on next launch just because
 * their phone is set to it.
 */

const STORAGE_KEY = "flipper_locale";

function initialLocale(): Locale {
  if (typeof localStorage !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "it" || saved === "en") return saved;
  }
  return detectLocale();
}

export default function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(initialLocale);

  const value = useMemo(
    () =>
      createI18n(locale, (next) => {
        setLocaleState(next);
        if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, next);
      }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
