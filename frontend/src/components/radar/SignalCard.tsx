import type { ActionId, Signal } from "../../types/signals";
import { availableActions, requiresTransmission, signalExplainKey, signalSummary } from "../../types/signals";
import { useT } from "../../i18n/useT";
import { rssiToPercent } from "./rssi";
import type { TranslationKey } from "../../i18n";

/**
 * One detected signal.
 *
 * Answers the three questions the Radar exists to answer, in that order: what
 * it is (in plain language), how many times it has been seen, and what can be
 * done with it.
 */

interface Props {
  signal: Signal;
  onAction: (signal: Signal, action: ActionId) => void;
}

function formatTimestamp(unixSeconds: number, locale: string): string {
  return new Date(unixSeconds * 1000).toLocaleString(locale);
}

export default function SignalCard({ signal, onAction }: Props) {
  const { t, tPlural, locale } = useT();
  const actions = availableActions(signal.kind);
  const explainKey = signalExplainKey(signal.kind);

  return (
    <article
      data-testid={`signal-${signal.id}`}
      className="rounded-xl border border-gray-700 bg-gray-800 p-4"
    >
      <header className="mb-2">
        <h3 className="text-base font-semibold text-white">{t(explainKey as TranslationKey)}</h3>
        <p className="font-mono text-xs text-gray-400">{signalSummary(signal.kind)}</p>
      </header>

      {/* The "explain the non-obvious" requirement: every signal says what it is
          before it says anything technical about itself. */}
      <p className="mb-3 text-sm text-gray-300">{t(`${explainKey}.detail` as TranslationKey)}</p>

      <dl className="mb-3 grid grid-cols-2 gap-2 text-xs">
        <div>
          <dt className="text-gray-500">{t("signal.last_seen")}</dt>
          <dd className="text-gray-300">{formatTimestamp(signal.last_seen, locale)}</dd>
        </div>
        <div>
          <dt className="text-gray-500">{t("signal.first_seen")}</dt>
          <dd className="text-gray-300">{formatTimestamp(signal.first_seen, locale)}</dd>
        </div>
      </dl>

      <p className="mb-3 text-xs font-medium text-emerald-400">
        {tPlural("signal.seen_count", signal.count)}
      </p>

      {signal.rssi_dbm !== null && (
        <div className="mb-3">
          <div className="mb-1 flex justify-between text-xs text-gray-500">
            <span>{t("signal.strength")}</span>
            <span className="tabular-nums">{signal.rssi_dbm} dBm</span>
          </div>
          <div
            className="h-1.5 w-full overflow-hidden rounded-full bg-gray-700"
            role="meter"
            aria-valuenow={signal.rssi_dbm}
            aria-valuemin={-100}
            aria-valuemax={-30}
            aria-label={t("signal.strength")}
          >
            <div
              className="h-full rounded-full bg-emerald-500"
              style={{ width: `${rssiToPercent(signal.rssi_dbm)}%` }}
            />
          </div>
          <p className="mt-1 text-[11px] text-gray-500">{t("signal.strength.hint")}</p>
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        {actions.map((action) => (
          <button
            key={action}
            type="button"
            onClick={() => onAction(signal, action)}
            data-action={action}
            title={t(`action.${action}.detail` as TranslationKey)}
            className={[
              "rounded-lg px-3 py-1.5 text-xs font-medium transition",
              "focus:outline-none focus:ring-2 focus:ring-white/70",
              // Transmitting actions are visually distinct because they are the
              // ones with legal weight behind them.
              requiresTransmission(action)
                ? "bg-amber-700 text-amber-50 hover:bg-amber-600"
                : "bg-gray-700 text-gray-200 hover:bg-gray-600",
            ].join(" ")}
          >
            {t(`action.${action}` as TranslationKey)}
          </button>
        ))}
      </div>
    </article>
  );
}
