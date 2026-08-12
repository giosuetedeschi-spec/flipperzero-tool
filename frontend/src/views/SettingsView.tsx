import { useT } from "../i18n/useT";
import { SUPPORTED_LOCALES, type Locale } from "../i18n";
import { useGeolocation } from "../hooks/useGeolocation";

/**
 * Settings: language, and the mock-device toggle.
 *
 * Mock mode lives here rather than in the header because it is a development
 * aid, not a mode users switch between. It stays reachable so the Radar can be
 * demonstrated with no Flipper attached.
 */

const LOCALE_NAMES: Record<Locale, string> = {
  it: "Italiano",
  en: "English",
};

interface Props {
  mockMode: boolean;
  onMockModeChange: (enabled: boolean) => void;
}

export default function SettingsView({ mockMode, onMockModeChange }: Props) {
  const { t, locale, setLocale } = useT();
  const geo = useGeolocation();

  return (
    <div className="flex flex-col gap-6 overflow-auto p-4" data-testid="settings-view">
      <header>
        <h1 className="text-xl font-bold text-white">{t("nav.settings")}</h1>
      </header>

      <section>
        <h2 className="mb-2 text-sm font-semibold text-gray-300">{t("settings.language")}</h2>
        <div className="flex gap-2">
          {SUPPORTED_LOCALES.map((candidate) => (
            <button
              key={candidate}
              type="button"
              onClick={() => setLocale(candidate)}
              aria-pressed={locale === candidate}
              data-locale={candidate}
              className={[
                "rounded-2xl px-4 py-2 text-sm font-medium transition",
                "focus:outline-none focus:ring-2 focus:ring-white/70",
                locale === candidate
                  ? "border border-emerald-500 bg-emerald-600 text-white"
                  : "border border-gray-600 bg-gray-800 text-gray-200 hover:border-gray-500",
              ].join(" ")}
            >
              {LOCALE_NAMES[candidate]}
            </button>
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-2 text-sm font-semibold text-gray-300">{t("settings.geotagging")}</h2>
        <button
          type="button"
          onClick={() => geo.setEnabled(!geo.enabled)}
          aria-pressed={geo.enabled}
          data-testid="geotagging-toggle"
          className={[
            "rounded-2xl px-4 py-2 text-sm font-medium transition",
            "focus:outline-none focus:ring-2 focus:ring-white/70",
            geo.enabled
              ? "border border-emerald-500 bg-emerald-600 text-white"
              : "border border-gray-600 bg-gray-800 text-gray-200 hover:border-gray-500",
          ].join(" ")}
        >
          {geo.enabled ? t("settings.geotagging.on") : t("settings.geotagging.off")}
        </button>
        <p className="mt-2 text-xs text-gray-500">{t("settings.geotagging.hint")}</p>
        {/* Say why there is no position rather than leaving the toggle looking
            broken -- indoors, where most tag reads happen, GPS routinely fails. */}
        {geo.enabled && geo.unavailableReason && (
          <p data-testid="geotagging-status" className="mt-1 text-xs text-amber-400">
            {t(`settings.geotagging.${geo.unavailableReason}` as never)}
          </p>
        )}
      </section>

      <section>
        <h2 className="mb-2 text-sm font-semibold text-gray-300">{t("settings.mock")}</h2>
        <button
          type="button"
          onClick={() => onMockModeChange(!mockMode)}
          aria-pressed={mockMode}
          data-testid="mock-toggle"
          className={[
            "rounded-2xl px-4 py-2 text-sm font-medium transition",
            "focus:outline-none focus:ring-2 focus:ring-white/70",
            mockMode
              ? "border border-purple-500 bg-purple-600 text-white"
              : "border border-gray-600 bg-gray-800 text-gray-200 hover:border-gray-500",
          ].join(" ")}
        >
          {mockMode ? t("settings.mock.on") : t("settings.mock.off")}
        </button>
        <p className="mt-2 text-xs text-gray-500">{t("settings.mock.hint")}</p>
      </section>
    </div>
  );
}
