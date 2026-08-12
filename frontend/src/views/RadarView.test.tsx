import { describe, expect, it as test, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { I18nContext, createI18n } from "../i18n/useT";
import RadarView from "./RadarView";

// The Radar talks to the backend through the IPC wrappers; in tests it runs in
// mock mode, which never reaches them, but the module still has to import.
vi.mock("../services/tauri", () => ({
  signalsListByChip: vi.fn(async () => []),
  signalsCountsByChip: vi.fn(async () => []),
}));

function withI18n(children: ReactNode, locale: "it" | "en" = "en") {
  return <I18nContext.Provider value={createI18n(locale, () => {})}>{children}</I18nContext.Provider>;
}

describe("RadarView", () => {
  beforeEach(() => vi.clearAllMocks());

  test("tells the user to connect when there is nothing to listen to", () => {
    render(withI18n(<RadarView connected={false} mockMode={false} />));
    expect(screen.getByTestId("radar-disconnected")).toBeInTheDocument();
  });

  test("shows the diagram with per-chip counts in mock mode", async () => {
    render(withI18n(<RadarView connected={false} mockMode />));

    expect(screen.getByTestId("flipper-schematic")).toBeInTheDocument();
    // Two demo Sub-GHz signals, one NFC.
    expect(await screen.findByTestId("count-subghz")).toHaveTextContent("2");
    expect(screen.getByTestId("count-nfc")).toHaveTextContent("1");
  });

  test("opens the chip sheet on tap and closes it again", async () => {
    render(withI18n(<RadarView connected={false} mockMode />));

    await userEvent.click(await screen.findByRole("button", { name: /Sub-GHz/ }));
    expect(screen.getByTestId("chip-sheet")).toHaveAttribute("data-chip", "subghz");

    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.queryByTestId("chip-sheet")).toBeNull();
  });

  test("a non-transmitting action runs immediately", async () => {
    const onPerform = vi.fn();
    render(withI18n(<RadarView connected={false} mockMode onPerform={onPerform} />));

    await userEvent.click(await screen.findByRole("button", { name: /Sub-GHz/ }));
    await userEvent.click(screen.getAllByRole("button", { name: "Analyse" })[0]);

    expect(screen.queryByTestId("transmit-gate")).toBeNull();
    expect(onPerform).toHaveBeenCalledOnce();
    expect(onPerform.mock.calls[0][1]).toBe("analyze");
  });

  test("a transmitting action is held behind the gate until confirmed", async () => {
    const onPerform = vi.fn();
    render(withI18n(<RadarView connected={false} mockMode onPerform={onPerform} />));

    await userEvent.click(await screen.findByRole("button", { name: /Sub-GHz/ }));
    await userEvent.click(screen.getAllByRole("button", { name: "Replay" })[0]);

    // Nothing has been performed yet: the warning has to be read first.
    expect(screen.getByTestId("transmit-gate")).toBeInTheDocument();
    expect(onPerform).not.toHaveBeenCalled();

    await userEvent.click(screen.getByTestId("transmit-confirm"));
    expect(onPerform).toHaveBeenCalledOnce();
    expect(onPerform.mock.calls[0][1]).toBe("replay");
    expect(screen.queryByTestId("transmit-gate")).toBeNull();
  });

  test("cancelling the gate transmits nothing", async () => {
    const onPerform = vi.fn();
    render(withI18n(<RadarView connected={false} mockMode onPerform={onPerform} />));

    await userEvent.click(await screen.findByRole("button", { name: /Sub-GHz/ }));
    await userEvent.click(screen.getAllByRole("button", { name: "Replay" })[0]);
    await userEvent.click(screen.getByTestId("transmit-cancel"));

    expect(onPerform).not.toHaveBeenCalled();
    expect(screen.queryByTestId("transmit-gate")).toBeNull();
  });

  test("over BLE, Bluetooth explains itself instead of showing an empty list", async () => {
    render(withI18n(<RadarView connected={false} mockMode transportIsBle />));

    await userEvent.click(await screen.findByRole("button", { name: /Bluetooth/ }));
    expect(screen.getByTestId("chip-unavailable")).toBeInTheDocument();
    expect(screen.queryByTestId("chip-empty")).toBeNull();
  });

  test("over USB, Bluetooth is a normal chip", async () => {
    render(withI18n(<RadarView connected={false} mockMode transportIsBle={false} />));

    await userEvent.click(await screen.findByRole("button", { name: /Bluetooth/ }));
    expect(screen.queryByTestId("chip-unavailable")).toBeNull();
    expect(screen.getByTestId("chip-empty")).toBeInTheDocument();
  });

  test("only the two demo Sub-GHz signals appear under Sub-GHz", async () => {
    render(withI18n(<RadarView connected={false} mockMode />));

    await userEvent.click(await screen.findByRole("button", { name: /Sub-GHz/ }));
    expect(screen.getByText("2 signals")).toBeInTheDocument();
    // The rolling-code demo signal must not offer a replay.
    expect(screen.getAllByRole("button", { name: "Replay" })).toHaveLength(1);
  });

  test("renders in Italian", async () => {
    render(withI18n(<RadarView connected={false} mockMode />, "it"));
    expect(screen.getByText("Cosa vede il Flipper")).toBeInTheDocument();
  });
});
