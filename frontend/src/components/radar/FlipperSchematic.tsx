import type { ChipSlug } from "../../types/signals";
import { isExternalModule } from "../../types/signals";
import { useT } from "../../i18n/useT";

/**
 * The interactive diagram of the Flipper: the Radar's front door.
 *
 * Chips are laid out roughly where they physically sit on the device, so the
 * picture teaches the hardware rather than being a menu wearing a costume. Each
 * hotspot is a real button -- not an SVG shape with a click handler -- so it is
 * reachable by keyboard and announced by screen readers.
 */

export type ChipAvailability = "active" | "unavailable" | "not_detected";

export interface ChipStatus {
  chip: ChipSlug;
  /** Distinct signals seen, not sightings. */
  count: number;
  availability: ChipAvailability;
}

interface Props {
  chips: ChipStatus[];
  selected: ChipSlug | null;
  onSelect: (chip: ChipSlug) => void;
}

/**
 * Hotspot positions as percentages of the diagram, matching where each radio
 * actually lives on the hardware.
 */
const LAYOUT: Record<ChipSlug, { x: number; y: number; label: string }> = {
  subghz: { x: 50, y: 8, label: "Sub-GHz" },
  infrared: { x: 82, y: 20, label: "IR" },
  nfc: { x: 22, y: 38, label: "NFC" },
  lfrfid: { x: 50, y: 38, label: "RFID" },
  ibutton: { x: 78, y: 38, label: "iButton" },
  bluetooth: { x: 22, y: 62, label: "Bluetooth" },
  gpio: { x: 50, y: 62, label: "GPIO" },
  wifi_devboard: { x: 20, y: 86, label: "WiFi" },
  external_cc1101: { x: 43, y: 86, label: "CC1101" },
  nrf24: { x: 66, y: 86, label: "NRF24" },
  unknown_module: { x: 87, y: 86, label: "?" },
};

/** Colour per chip, so a signal's origin is recognisable without reading. */
const ACCENT: Record<ChipSlug, string> = {
  subghz: "bg-emerald-600 border-emerald-400",
  nfc: "bg-sky-600 border-sky-400",
  lfrfid: "bg-indigo-600 border-indigo-400",
  ibutton: "bg-amber-600 border-amber-400",
  infrared: "bg-rose-600 border-rose-400",
  bluetooth: "bg-blue-600 border-blue-400",
  gpio: "bg-slate-600 border-slate-400",
  wifi_devboard: "bg-teal-600 border-teal-400",
  external_cc1101: "bg-lime-600 border-lime-400",
  nrf24: "bg-fuchsia-600 border-fuchsia-400",
  unknown_module: "bg-gray-600 border-gray-400",
};

export default function FlipperSchematic({ chips, selected, onSelect }: Props) {
  const { t } = useT();

  return (
    <div
      className="relative mx-auto w-full max-w-md aspect-[3/4] rounded-2xl border border-gray-700 bg-gray-900"
      role="group"
      aria-label={t("radar.title")}
      data-testid="flipper-schematic"
    >
      {chips.map(({ chip, count, availability }) => {
        const position = LAYOUT[chip];
        const isSelected = selected === chip;
        const isDimmed = availability !== "active";

        // An external module that was never detected is not drawn at all --
        // showing an empty GPIO header's worth of phantom hardware would imply
        // the user has boards they do not own.
        if (availability === "not_detected" && isExternalModule(chip)) return null;

        return (
          <button
            key={chip}
            type="button"
            onClick={() => onSelect(chip)}
            aria-pressed={isSelected}
            data-chip={chip}
            data-availability={availability}
            className={[
              "absolute -translate-x-1/2 -translate-y-1/2",
              "flex flex-col items-center gap-1 rounded-xl border-2 px-3 py-2",
              "text-xs font-medium text-white transition",
              "focus:outline-none focus:ring-2 focus:ring-white/70",
              isDimmed ? "border-gray-600 bg-gray-800 opacity-60" : ACCENT[chip],
              isSelected ? "ring-2 ring-white scale-105" : "",
            ].join(" ")}
            style={{ left: `${position.x}%`, top: `${position.y}%` }}
          >
            <span>{position.label}</span>

            {availability === "active" && count > 0 && (
              <span
                data-testid={`count-${chip}`}
                className="rounded-full bg-black/50 px-2 py-0.5 text-[10px] tabular-nums"
              >
                {count}
              </span>
            )}

            {availability === "unavailable" && (
              <span className="text-[10px] opacity-80">{t("radar.chip.unavailable")}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}
