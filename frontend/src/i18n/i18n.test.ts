import { describe, expect, it as test } from "vitest";
import { it } from "./it";
import { en } from "./en";
import { DEFAULT_LOCALE, resolveLocale, translate, translatePlural } from "./index";
import {
  availableActions,
  chipExplainKey,
  requiresTransmission,
  signalExplainKey,
  signalSummary,
  type ChipSlug,
} from "../types/signals";

const ALL_CHIPS: ChipSlug[] = [
  "subghz",
  "nfc",
  "lfrfid",
  "ibutton",
  "infrared",
  "bluetooth",
  "gpio",
  "wifi_devboard",
  "external_cc1101",
  "nrf24",
  "unknown_module",
];

describe("translation catalogues", () => {
  test("both locales define exactly the same keys", () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(it).sort());
  });

  test("no translation is left empty", () => {
    for (const [locale, catalogue] of Object.entries({ it, en })) {
      for (const [key, value] of Object.entries(catalogue)) {
        expect(value.trim(), `${locale}:${key} is empty`).not.toBe("");
      }
    }
  });

  test("every chip has a name and a beginner-facing explanation in both locales", () => {
    // The whole premise is that someone who knows nothing can use this, so a
    // chip without a detail string is a broken feature, not a cosmetic gap.
    for (const chip of ALL_CHIPS) {
      const key = chipExplainKey(chip);
      for (const locale of ["it", "en"] as const) {
        expect(translate(locale, key as never), `${locale}:${key}`).not.toBe(key);
        expect(translate(locale, `${key}.detail` as never)).not.toBe(`${key}.detail`);
      }
    }
  });

  test("every signal explanation key resolves in both locales", () => {
    const kinds = [
      { type: "sub_ghz", frequency_hz: 433920000, preset: null, protocol: null, rolling_code: true },
      { type: "sub_ghz", frequency_hz: 433920000, preset: null, protocol: null, rolling_code: false },
      { type: "sub_ghz", frequency_hz: 433920000, preset: null, protocol: null, rolling_code: null },
      { type: "nfc", technology: "ISO14443-3A", uid: "AA", atqa: null, sak: null },
      { type: "lf_rfid", protocol: "EM4100", data: "01" },
      { type: "i_button", protocol: "DS1990", id: "01" },
      { type: "infrared", protocol: "NEC", address: "00", command: "01" },
      { type: "bluetooth", address: "AA:BB", name: null },
      { type: "gpio", pin: "PA7", level: true },
      { type: "module", module: "esp32", detail: null },
    ] as const;

    for (const kind of kinds) {
      const key = signalExplainKey(kind);
      for (const locale of ["it", "en"] as const) {
        expect(translate(locale, key as never), `${locale}:${key}`).not.toBe(key);
        expect(translate(locale, `${key}.detail` as never)).not.toBe(`${key}.detail`);
      }
    }
  });

  test("every action has a label and an explanation in both locales", () => {
    for (const action of ["save", "replay", "emulate", "analyze", "compare", "export"] as const) {
      for (const locale of ["it", "en"] as const) {
        expect(translate(locale, `action.${action}` as never)).not.toBe(`action.${action}`);
        expect(translate(locale, `action.${action}.detail` as never)).not.toBe(
          `action.${action}.detail`,
        );
      }
    }
  });
});

describe("translate", () => {
  test("substitutes named placeholders", () => {
    expect(translate("en", "radar.signals.count_other", { count: 4 })).toBe("4 signals");
  });

  test("leaves unknown placeholders untouched rather than blanking them", () => {
    expect(translate("en", "radar.signals.count_other", {})).toContain("{count}");
  });

  test("falls back to the key so a missing string stays visible", () => {
    expect(translate("en", "does.not.exist" as never)).toBe("does.not.exist");
  });
});

describe("translatePlural", () => {
  test("uses the singular only for exactly one", () => {
    expect(translatePlural("en", "signal.seen_count", 1)).toBe("Seen 1 time");
    expect(translatePlural("en", "signal.seen_count", 2)).toBe("Seen 2 times");
    expect(translatePlural("en", "signal.seen_count", 0)).toBe("Seen 0 times");
  });

  test("works in Italian too", () => {
    expect(translatePlural("it", "signal.seen_count", 1)).toBe("Visto 1 volta");
    expect(translatePlural("it", "signal.seen_count", 3)).toBe("Visto 3 volte");
  });
});

describe("resolveLocale", () => {
  test("matches on the primary subtag", () => {
    expect(resolveLocale("it-CH")).toBe("it");
    expect(resolveLocale("en-GB")).toBe("en");
  });

  test("falls back for unsupported or missing languages", () => {
    expect(resolveLocale("de-DE")).toBe(DEFAULT_LOCALE);
    expect(resolveLocale(undefined)).toBe(DEFAULT_LOCALE);
    expect(resolveLocale("")).toBe(DEFAULT_LOCALE);
  });
});

describe("signal helpers mirror the Rust model", () => {
  test("a rolling-code remote is never offered a replay", () => {
    const rolling = {
      type: "sub_ghz",
      frequency_hz: 433920000,
      preset: null,
      protocol: null,
      rolling_code: true,
    } as const;
    expect(availableActions(rolling)).not.toContain("replay");

    const fixed = { ...rolling, rolling_code: false } as const;
    expect(availableActions(fixed)).toContain("replay");
  });

  test("observation-only signals offer nothing that transmits", () => {
    const kinds = [
      { type: "bluetooth", address: "AA:BB", name: null },
      { type: "gpio", pin: "PA7", level: false },
      { type: "module", module: "esp32", detail: null },
    ] as const;

    for (const kind of kinds) {
      expect(availableActions(kind).some(requiresTransmission)).toBe(false);
    }
  });

  test("tags are emulated, never replayed", () => {
    const nfc = { type: "nfc", technology: "ISO14443-3A", uid: "AA", atqa: null, sak: null } as const;
    expect(availableActions(nfc)).toContain("emulate");
    expect(availableActions(nfc)).not.toContain("replay");
  });

  test("only replay and emulate are gated as transmissions", () => {
    expect(requiresTransmission("replay")).toBe(true);
    expect(requiresTransmission("emulate")).toBe(true);
    for (const action of ["save", "analyze", "compare", "export"] as const) {
      expect(requiresTransmission(action)).toBe(false);
    }
  });

  test("summaries show identifying detail so signals are distinguishable", () => {
    expect(
      signalSummary({
        type: "sub_ghz",
        frequency_hz: 433920000,
        preset: null,
        protocol: "Princeton",
        rolling_code: false,
      }),
    ).toBe("433.92 MHz · Princeton");

    expect(
      signalSummary({ type: "nfc", technology: "ISO14443-3A", uid: "04A2B3", atqa: null, sak: null }),
    ).toBe("ISO14443-3A · 04A2B3");
  });
});
