import { describe, expect, it as test, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { I18nContext, createI18n } from "../i18n/useT";
import AppShell from "./AppShell";
import { SECTIONS } from "./sections";

/**
 * jsdom has no real media queries, so viewport width is faked per test. Without
 * this every test would see the desktop layout and the mobile nav would never
 * be exercised.
 */
function setViewport(isMobile: boolean) {
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: isMobile,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  }));
}

function withI18n(children: ReactNode, locale: "it" | "en" = "en") {
  return <I18nContext.Provider value={createI18n(locale, () => {})}>{children}</I18nContext.Provider>;
}

describe("AppShell", () => {
  beforeEach(() => vi.unstubAllGlobals());

  test("shows the bottom tab bar on a phone and the top bar on desktop", () => {
    setViewport(true);
    const { unmount } = render(
      withI18n(
        <AppShell section="radar" onNavigate={() => {}}>
          <p>content</p>
        </AppShell>,
      ),
    );
    expect(screen.getByTestId("mobile-nav")).toBeInTheDocument();
    expect(screen.queryByTestId("desktop-nav")).toBeNull();
    unmount();

    setViewport(false);
    render(
      withI18n(
        <AppShell section="radar" onNavigate={() => {}}>
          <p>content</p>
        </AppShell>,
      ),
    );
    expect(screen.getByTestId("desktop-nav")).toBeInTheDocument();
    expect(screen.queryByTestId("mobile-nav")).toBeNull();
  });

  test("every section is reachable from the navigation", () => {
    // The regression this guards: components existing but nothing rendering
    // them. If a section is dropped from the bar, it is unreachable again.
    setViewport(false);
    render(
      withI18n(
        <AppShell section="files" onNavigate={() => {}}>
          <p>content</p>
        </AppShell>,
      ),
    );

    for (const section of SECTIONS) {
      expect(
        document.querySelector(`[data-section="${section}"]`),
        `${section} has no tab`,
      ).toBeInTheDocument();
    }
  });

  test("marks the current section for assistive technology", () => {
    setViewport(false);
    render(
      withI18n(
        <AppShell section="tools" onNavigate={() => {}}>
          <p>content</p>
        </AppShell>,
      ),
    );
    expect(document.querySelector('[data-section="tools"]')).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(document.querySelector('[data-section="files"]')).not.toHaveAttribute("aria-current");
  });

  test("tapping a tab reports the new section", async () => {
    setViewport(true);
    const onNavigate = vi.fn();
    render(
      withI18n(
        <AppShell section="radar" onNavigate={onNavigate}>
          <p>content</p>
        </AppShell>,
      ),
    );

    await userEvent.click(screen.getByRole("button", { name: /Files/ }));
    expect(onNavigate).toHaveBeenCalledWith("files");
  });

  test("renders its children", () => {
    setViewport(false);
    render(
      withI18n(
        <AppShell section="radar" onNavigate={() => {}}>
          <p>the section content</p>
        </AppShell>,
      ),
    );
    expect(screen.getByText("the section content")).toBeInTheDocument();
  });

  test("navigation labels translate", () => {
    setViewport(false);
    render(
      withI18n(
        <AppShell section="radar" onNavigate={() => {}}>
          <p>content</p>
        </AppShell>,
        "it",
      ),
    );
    expect(screen.getByRole("button", { name: /Strumenti/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Impostazioni/ })).toBeInTheDocument();
  });
});
