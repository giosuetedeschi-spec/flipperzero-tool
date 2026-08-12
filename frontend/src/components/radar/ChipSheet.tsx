import type { ActionId, ChipSlug, Signal } from "../../types/signals";
import { chipExplainKey, isExternalModule } from "../../types/signals";
import { useT } from "../../i18n/useT";
import type { TranslationKey } from "../../i18n";
import SignalCard from "./SignalCard";
import type { ChipAvailability } from "./FlipperSchematic";

/**
 * The panel that opens when a chip is tapped.
 *
 * Leads with what the chip *is* before listing anything it found, because the
 * user this app is designed for may not know what "Sub-GHz" means, and a list
 * of frequencies would tell them nothing.
 */

interface Props {
  chip: ChipSlug;
  signals: Signal[];
  availability: ChipAvailability;
  onClose: () => void;
  onAction: (signal: Signal, action: ActionId) => void;
}

export default function ChipSheet({ chip, signals, availability, onClose, onAction }: Props) {
  const { t, tPlural } = useT();
  const explainKey = chipExplainKey(chip);

  return (
    <section
      data-testid="chip-sheet"
      data-chip={chip}
      aria-label={t(explainKey as TranslationKey)}
      className="rounded-t-2xl border-t border-gray-700 bg-gray-900 p-4"
    >
      <header className="mb-3 flex items-start justify-between gap-4">
        <div>
          <h2 className="text-lg font-semibold text-white">{t(explainKey as TranslationKey)}</h2>
          <p className="mt-1 text-sm text-gray-400">
            {t(`${explainKey}.detail` as TranslationKey)}
          </p>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label={t("common.close")}
          className="rounded-lg bg-gray-800 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-white/70"
        >
          {t("common.close")}
        </button>
      </header>

      {/* Say plainly why a chip is dark, rather than showing an empty list that
          looks identical to "nothing is nearby". */}
      {availability === "unavailable" && (
        <p
          data-testid="chip-unavailable"
          className="rounded-lg border border-amber-700 bg-amber-950 p-3 text-sm text-amber-200"
        >
          {t("radar.chip.unavailable.ble")}
        </p>
      )}

      {availability === "not_detected" && (
        <p
          data-testid="chip-not-detected"
          className="rounded-lg border border-gray-700 bg-gray-800 p-3 text-sm text-gray-300"
        >
          {t(isExternalModule(chip) ? "radar.chip.not_detected.hint" : "radar.empty.hint")}
        </p>
      )}

      {availability === "active" && signals.length === 0 && (
        <div data-testid="chip-empty" className="rounded-lg bg-gray-800 p-4 text-center">
          <p className="text-sm font-medium text-gray-300">{t("radar.empty")}</p>
          <p className="mt-1 text-xs text-gray-500">{t("radar.empty.hint")}</p>
        </div>
      )}

      {availability === "active" && signals.length > 0 && (
        <>
          <p className="mb-2 text-xs font-medium text-emerald-400">
            {tPlural("radar.signals.count", signals.length)}
          </p>
          <ul className="flex flex-col gap-3">
            {signals.map((signal) => (
              <li key={signal.id}>
                <SignalCard signal={signal} onAction={onAction} />
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
