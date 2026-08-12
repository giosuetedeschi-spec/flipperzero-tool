import type { ReactNode } from "react";
import { useT } from "../i18n/useT";
import type { TranslationKey } from "../i18n";
import { useIsMobileLayout } from "./useBreakpoint";
import { SECTIONS, type Section } from "./sections";

/**
 * The navigation frame around every screen.
 *
 * One component serves both shapes rather than two parallel trees: a bottom tab
 * bar on a phone, a horizontal bar on desktop. Splitting them would mean every
 * new section had to be added twice, and one of the two would drift.
 */

const LABEL_KEYS: Record<Section, TranslationKey> = {
  radar: "nav.radar",
  files: "nav.files",
  tools: "nav.tools",
  device: "nav.device",
  settings: "nav.settings",
};

const ICONS: Record<Section, string> = {
  radar: "◎",
  files: "▤",
  tools: "⚙",
  device: "▮",
  settings: "⋯",
};

interface Props {
  section: Section;
  onNavigate: (section: Section) => void;
  children: ReactNode;
}

export default function AppShell({ section, onNavigate, children }: Props) {
  const { t } = useT();
  const isMobile = useIsMobileLayout();

  const tabs = SECTIONS.map((candidate) => {
    const active = candidate === section;
    return (
      <button
        key={candidate}
        type="button"
        onClick={() => onNavigate(candidate)}
        aria-current={active ? "page" : undefined}
        data-section={candidate}
        className={[
          "flex items-center justify-center gap-2 transition",
          "focus:outline-none focus:ring-2 focus:ring-white/70",
          isMobile ? "flex-1 flex-col py-2 text-[11px]" : "rounded-2xl px-4 py-2 text-sm",
          active
            ? isMobile
              ? "text-emerald-400"
              : "border border-emerald-500 bg-emerald-600 font-semibold text-white"
            : isMobile
              ? "text-gray-500"
              : "border border-gray-600 bg-gray-800 text-gray-200 hover:border-gray-500 hover:text-white",
        ].join(" ")}
      >
        <span aria-hidden="true" className={isMobile ? "text-lg leading-none" : ""}>
          {ICONS[candidate]}
        </span>
        <span>{t(LABEL_KEYS[candidate])}</span>
      </button>
    );
  });

  return (
    <div className="flex min-h-screen flex-col bg-gray-900 text-gray-100" data-testid="app-shell">
      {!isMobile && (
        <nav
          aria-label={t("nav.sections")}
          data-testid="desktop-nav"
          className="flex items-center gap-2 border-b border-gray-700 bg-gray-800 px-4 py-2"
        >
          <span className="mr-2 text-lg font-bold text-emerald-400">{t("app.name")}</span>
          {tabs}
        </nav>
      )}

      <main className="flex flex-1 flex-col overflow-hidden">{children}</main>

      {isMobile && (
        <nav
          aria-label={t("nav.sections")}
          data-testid="mobile-nav"
          // Padded for the home indicator so the last tab is not under it.
          className="flex border-t border-gray-700 bg-gray-800 pb-[env(safe-area-inset-bottom)]"
        >
          {tabs}
        </nav>
      )}
    </div>
  );
}
