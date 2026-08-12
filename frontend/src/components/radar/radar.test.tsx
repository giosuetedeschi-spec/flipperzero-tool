import { beforeEach, describe, expect, it as test, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { I18nContext, createI18n } from "../../i18n/useT";
import FlipperSchematic, { type ChipStatus } from "./FlipperSchematic";
import ChipSheet from "./ChipSheet";
import SignalCard from "./SignalCard";
import { rssiToPercent } from "./rssi";
import TransmitGate from "./TransmitGate";
import RadarOnboarding from "./RadarOnboarding";
import { hasSeenOnboarding } from "./onboardingState";
import type { Signal } from "../../types/signals";

function withI18n(children: ReactNode, locale: "it" | "en" = "en") {
  return <I18nContext.Provider value={createI18n(locale, () => {})}>{children}</I18nContext.Provider>;
}

function remote(overrides: Partial<Signal> = {}): Signal {
  return {
    id: "subghz-abc",
    chip: "subghz",
    kind: {
      type: "sub_ghz",
      frequency_hz: 433_920_000,
      preset: "AM650",
      protocol: "Princeton",
      rolling_code: false,
    },
    first_seen: 1_700_000_000,
    last_seen: 1_700_000_900,
    count: 3,
    rssi_dbm: -62,
    location: null,
    ...overrides,
  };
}

describe("FlipperSchematic", () => {
  const chips: ChipStatus[] = [
    { chip: "subghz", count: 3, availability: "active" },
    { chip: "nfc", count: 0, availability: "active" },
    { chip: "bluetooth", count: 0, availability: "unavailable" },
    { chip: "nrf24", count: 0, availability: "not_detected" },
  ];

  test("shows a count badge only where signals were actually seen", () => {
    render(withI18n(<FlipperSchematic chips={chips} selected={null} onSelect={() => {}} />));

    expect(screen.getByTestId("count-subghz")).toHaveTextContent("3");
    expect(screen.queryByTestId("count-nfc")).toBeNull();
  });

  test("does not draw external modules that were never detected", () => {
    // Drawing them would imply the user owns add-on boards they do not have.
    render(withI18n(<FlipperSchematic chips={chips} selected={null} onSelect={() => {}} />));
    expect(document.querySelector('[data-chip="nrf24"]')).toBeNull();
  });

  test("marks an unavailable chip instead of showing it as simply empty", () => {
    render(withI18n(<FlipperSchematic chips={chips} selected={null} onSelect={() => {}} />));
    const bluetooth = document.querySelector('[data-chip="bluetooth"]');
    expect(bluetooth).toHaveAttribute("data-availability", "unavailable");
  });

  test("hotspots are real buttons, so they are keyboard reachable", async () => {
    const onSelect = vi.fn();
    render(withI18n(<FlipperSchematic chips={chips} selected={null} onSelect={onSelect} />));

    const subghz = screen.getByRole("button", { name: /Sub-GHz/ });
    subghz.focus();
    await userEvent.keyboard("{Enter}");

    expect(onSelect).toHaveBeenCalledWith("subghz");
  });

  test("reports the selected chip through aria-pressed", () => {
    render(withI18n(<FlipperSchematic chips={chips} selected="subghz" onSelect={() => {}} />));
    expect(document.querySelector('[data-chip="subghz"]')).toHaveAttribute("aria-pressed", "true");
    expect(document.querySelector('[data-chip="nfc"]')).toHaveAttribute("aria-pressed", "false");
  });
});

describe("ChipSheet", () => {
  test("explains what the chip is before listing anything it found", () => {
    render(
      withI18n(
        <ChipSheet
          chip="subghz"
          signals={[remote()]}
          availability="active"
          onClose={() => {}}
          onAction={() => {}}
        />,
      ),
    );

    expect(screen.getByText("Long-range radio")).toBeInTheDocument();
    expect(screen.getByText(/gate and garage remotes/)).toBeInTheDocument();
  });

  test("says why Bluetooth is dark rather than showing an empty list", () => {
    // An empty list reads as "nothing nearby", which would be a lie.
    render(
      withI18n(
        <ChipSheet
          chip="bluetooth"
          signals={[]}
          availability="unavailable"
          onClose={() => {}}
          onAction={() => {}}
        />,
      ),
    );

    expect(screen.getByTestId("chip-unavailable")).toHaveTextContent(
      /busy talking to this phone/,
    );
    expect(screen.queryByTestId("chip-empty")).toBeNull();
  });

  test("an active chip with nothing found gets an encouraging hint", () => {
    render(
      withI18n(
        <ChipSheet
          chip="nfc"
          signals={[]}
          availability="active"
          onClose={() => {}}
          onAction={() => {}}
        />,
      ),
    );
    expect(screen.getByTestId("chip-empty")).toBeInTheDocument();
  });

  test("counts the signals it is showing", () => {
    render(
      withI18n(
        <ChipSheet
          chip="subghz"
          signals={[remote(), remote({ id: "subghz-def" })]}
          availability="active"
          onClose={() => {}}
          onAction={() => {}}
        />,
      ),
    );
    expect(screen.getByText("2 signals")).toBeInTheDocument();
  });
});

describe("SignalCard", () => {
  test("answers what it is, how many times, and what can be done", () => {
    render(withI18n(<SignalCard signal={remote()} onAction={() => {}} />));

    expect(screen.getByText("Fixed-code remote")).toBeInTheDocument();
    expect(screen.getByText("Seen 3 times")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Replay" })).toBeInTheDocument();
  });

  test("shows the identifying detail, not a generic label", () => {
    render(withI18n(<SignalCard signal={remote()} onAction={() => {}} />));
    expect(screen.getByText("433.92 MHz · Princeton")).toBeInTheDocument();
  });

  test("a rolling-code remote offers no replay and explains why", () => {
    const rolling = remote({
      kind: {
        type: "sub_ghz",
        frequency_hz: 433_920_000,
        preset: null,
        protocol: null,
        rolling_code: true,
      },
    });
    render(withI18n(<SignalCard signal={rolling} onAction={() => {}} />));

    expect(screen.queryByRole("button", { name: "Replay" })).toBeNull();
    expect(screen.getByText(/re-transmitting it would open nothing/)).toBeInTheDocument();
  });

  test("reports signal strength as an accessible meter", () => {
    render(withI18n(<SignalCard signal={remote()} onAction={() => {}} />));
    const meter = screen.getByRole("meter");
    expect(meter).toHaveAttribute("aria-valuenow", "-62");
  });

  test("omits the strength meter when there is no reading", () => {
    render(withI18n(<SignalCard signal={remote({ rssi_dbm: null })} onAction={() => {}} />));
    expect(screen.queryByRole("meter")).toBeNull();
  });

  test("passes the chosen action up", async () => {
    const onAction = vi.fn();
    const signal = remote();
    render(withI18n(<SignalCard signal={signal} onAction={onAction} />));

    await userEvent.click(screen.getByRole("button", { name: "Replay" }));
    expect(onAction).toHaveBeenCalledWith(signal, "replay");
  });

  test("renders in Italian when the locale says so", () => {
    render(withI18n(<SignalCard signal={remote()} onAction={() => {}} />, "it"));
    expect(screen.getByText("Telecomando a codice fisso")).toBeInTheDocument();
    expect(screen.getByText("Visto 3 volte")).toBeInTheDocument();
  });
});

describe("rssiToPercent", () => {
  test("maps the useful range across the full bar", () => {
    expect(rssiToPercent(-100)).toBe(0);
    expect(rssiToPercent(-30)).toBe(100);
    expect(rssiToPercent(-65)).toBe(50);
  });

  test("clamps readings outside the useful range", () => {
    // Values beyond the clamp would otherwise overflow or underflow the bar.
    expect(rssiToPercent(-120)).toBe(0);
    expect(rssiToPercent(0)).toBe(100);
  });
});

describe("TransmitGate", () => {
  test("blocks with an explicit warning before anything is transmitted", () => {
    render(withI18n(<TransmitGate action="replay" onConfirm={() => {}} onCancel={() => {}} />));

    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(screen.getByText(/illegal in most countries/)).toBeInTheDocument();
  });

  test("confirming and cancelling are both explicit choices", async () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(withI18n(<TransmitGate action="emulate" onConfirm={onConfirm} onCancel={onCancel} />));

    await userEvent.click(screen.getByTestId("transmit-confirm"));
    expect(onConfirm).toHaveBeenCalledOnce();

    await userEvent.click(screen.getByTestId("transmit-cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });
});

describe("RadarOnboarding", () => {
  beforeEach(() => localStorage.clear());

  test("explains what the Radar is before the user meets the diagram", () => {
    render(withI18n(<RadarOnboarding onDismiss={() => {}} />));
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("This is what your Flipper can hear")).toBeInTheDocument();
  });

  test("walks through every step and ends on the legal one", async () => {
    render(withI18n(<RadarOnboarding onDismiss={() => {}} />));

    await userEvent.click(screen.getByTestId("onboarding-next"));
    expect(screen.getByText("The numbers count distinct things")).toBeInTheDocument();

    await userEvent.click(screen.getByTestId("onboarding-next"));
    // The legal limits are stated before the user ever meets a transmit button.
    expect(screen.getByText(/illegal/)).toBeInTheDocument();
  });

  test("finishing marks it seen so it never reappears", async () => {
    const onDismiss = vi.fn();
    render(withI18n(<RadarOnboarding onDismiss={onDismiss} />));

    await userEvent.click(screen.getByTestId("onboarding-next"));
    await userEvent.click(screen.getByTestId("onboarding-next"));
    await userEvent.click(screen.getByTestId("onboarding-next"));

    expect(onDismiss).toHaveBeenCalledOnce();
    expect(hasSeenOnboarding()).toBe(true);
  });

  test("skipping is as final as finishing", async () => {
    // An explainer that comes back after being dismissed is an irritation.
    const onDismiss = vi.fn();
    render(withI18n(<RadarOnboarding onDismiss={onDismiss} />));

    await userEvent.click(screen.getByTestId("onboarding-skip"));
    expect(onDismiss).toHaveBeenCalledOnce();
    expect(hasSeenOnboarding()).toBe(true);
  });

  test("translates", () => {
    render(withI18n(<RadarOnboarding onDismiss={() => {}} />, "it"));
    expect(screen.getByText("Questo è ciò che il Flipper sente")).toBeInTheDocument();
  });
});
