import { useState } from "react";
import { useT } from "../../i18n/useT";
import type { TranslationKey } from "../../i18n";
import { markOnboardingSeen } from "./onboardingState";

/**
 * A first-run explanation of what the Radar is.
 *
 * The app is meant to be usable by someone who has never heard of Sub-GHz or a
 * rolling code. Dropping that person straight onto a diagram of eleven radios
 * teaches them nothing, so three short cards say what they are looking at, what
 * the numbers mean, and where the limits are -- including that transmitting is
 * their legal responsibility, stated before they ever meet the button.
 *
 * Dismissed permanently once read. An explainer that reappears is an
 * irritation, not a help.
 */

const STEPS: { titleKey: TranslationKey; bodyKey: TranslationKey; icon: string }[] = [
  { titleKey: "onboarding.what.title", bodyKey: "onboarding.what.body", icon: "◎" },
  { titleKey: "onboarding.counts.title", bodyKey: "onboarding.counts.body", icon: "▣" },
  { titleKey: "onboarding.limits.title", bodyKey: "onboarding.limits.body", icon: "⚠" },
];

interface Props {
  onDismiss: () => void;
}

export default function RadarOnboarding({ onDismiss }: Props) {
  const { t } = useT();
  const [step, setStep] = useState(0);
  const isLast = step === STEPS.length - 1;
  const current = STEPS[step];

  const finish = () => {
    markOnboardingSeen();
    onDismiss();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="onboarding-title"
      data-testid="radar-onboarding"
      className="fixed inset-0 z-40 flex items-end justify-center bg-black/70 p-4 sm:items-center"
    >
      <div className="w-full max-w-md rounded-2xl border border-gray-700 bg-gray-900 p-5">
        <div className="mb-3 text-3xl" aria-hidden="true">
          {current.icon}
        </div>

        <h2 id="onboarding-title" className="text-lg font-semibold text-white">
          {t(current.titleKey)}
        </h2>
        <p className="mt-2 text-sm text-gray-300">{t(current.bodyKey)}</p>

        <div className="mt-5 flex items-center justify-between gap-4">
          <div className="flex gap-1.5" aria-hidden="true">
            {STEPS.map((_, index) => (
              <span
                key={index}
                className={`h-1.5 w-1.5 rounded-full ${
                  index === step ? "bg-emerald-400" : "bg-gray-600"
                }`}
              />
            ))}
          </div>

          <div className="flex gap-2">
            {/* Skipping is as final as finishing: someone who does not want the
                tour should not be shown it again either. */}
            <button
              type="button"
              onClick={finish}
              data-testid="onboarding-skip"
              className="rounded-lg px-3 py-2 text-sm text-gray-400 hover:text-gray-200 focus:outline-none focus:ring-2 focus:ring-white/70"
            >
              {t("onboarding.skip")}
            </button>
            <button
              type="button"
              onClick={() => (isLast ? finish() : setStep(step + 1))}
              data-testid="onboarding-next"
              className="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500 focus:outline-none focus:ring-2 focus:ring-white/70"
            >
              {isLast ? t("onboarding.start") : t("onboarding.next")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
