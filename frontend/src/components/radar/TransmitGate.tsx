import { useT } from "../../i18n/useT";
import type { TranslationKey } from "../../i18n";
import type { ActionId } from "../../types/signals";

/**
 * The confirmation shown before the Flipper transmits anything.
 *
 * Gated on the action rather than the chip, because transmission is what
 * carries legal weight. Deliberately a blocking dialog with an explicit
 * affirmative button: a toast or a checkbox buried in settings would let a user
 * transmit without ever having read the warning.
 */

interface Props {
  action: ActionId;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function TransmitGate({ action, onConfirm, onCancel }: Props) {
  const { t } = useT();

  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="transmit-gate-title"
      data-testid="transmit-gate"
      className="fixed inset-0 z-50 flex items-end justify-center bg-black/70 p-4 sm:items-center"
    >
      <div className="w-full max-w-md rounded-2xl border border-amber-700 bg-gray-900 p-5">
        <h2 id="transmit-gate-title" className="text-lg font-semibold text-amber-300">
          {t("legal.title")}
        </h2>

        <p className="mt-2 text-sm text-gray-300">{t("legal.body")}</p>

        <p className="mt-3 rounded-lg bg-gray-800 p-3 text-sm text-gray-200">
          {t(`action.${action}.detail` as TranslationKey)}
        </p>

        <div className="mt-4 flex flex-col gap-2">
          <button
            type="button"
            onClick={onConfirm}
            data-testid="transmit-confirm"
            className="rounded-lg bg-amber-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-amber-500 focus:outline-none focus:ring-2 focus:ring-white/70"
          >
            {t("legal.confirm")}
          </button>
          <button
            type="button"
            onClick={onCancel}
            data-testid="transmit-cancel"
            className="rounded-lg bg-gray-700 px-4 py-2.5 text-sm font-medium text-gray-200 hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-white/70"
          >
            {t("legal.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
