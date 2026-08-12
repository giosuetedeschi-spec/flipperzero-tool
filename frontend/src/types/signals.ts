/**
 * TypeScript mirrors of the Rust types in `src-tauri/src/signals/model.rs`.
 *
 * Field names match the serde representation exactly, so anything crossing the
 * IPC boundary needs no translation layer.
 */

export type ChipSlug =
  | "subghz"
  | "nfc"
  | "lfrfid"
  | "ibutton"
  | "infrared"
  | "bluetooth"
  | "gpio"
  | "wifi_devboard"
  | "external_cc1101"
  | "nrf24"
  | "unknown_module";

export type SignalKind =
  | {
      type: "sub_ghz";
      frequency_hz: number;
      preset: string | null;
      protocol: string | null;
      rolling_code: boolean | null;
    }
  | { type: "nfc"; technology: string; uid: string; atqa: string | null; sak: string | null }
  | { type: "lf_rfid"; protocol: string; data: string }
  | { type: "i_button"; protocol: string; id: string }
  | {
      type: "infrared";
      protocol: string | null;
      address: string | null;
      command: string | null;
    }
  | { type: "bluetooth"; address: string; name: string | null }
  | { type: "gpio"; pin: string; level: boolean }
  | { type: "module"; module: string; detail: string | null };

export interface GeoPoint {
  latitude: number;
  longitude: number;
}

export type ActionId = "save" | "replay" | "emulate" | "analyze" | "compare" | "export";

/** Actions that make the Flipper transmit, and therefore need the legal gate. */
const TRANSMITTING_ACTIONS: ReadonlySet<ActionId> = new Set<ActionId>(["replay", "emulate"]);

export function requiresTransmission(action: ActionId): boolean {
  return TRANSMITTING_ACTIONS.has(action);
}

export interface Signal {
  id: string;
  chip: ChipSlug;
  kind: SignalKind;
  /** Unix seconds. */
  first_seen: number;
  last_seen: number;
  count: number;
  rssi_dbm: number | null;
  location: GeoPoint | null;
  raw?: number[];
}

/** Chips that plug into the GPIO header rather than being built in. */
const EXTERNAL_MODULES: ReadonlySet<ChipSlug> = new Set<ChipSlug>([
  "wifi_devboard",
  "external_cc1101",
  "nrf24",
  "unknown_module",
]);

export function isExternalModule(chip: ChipSlug): boolean {
  return EXTERNAL_MODULES.has(chip);
}

/** Translation key for a chip's explanation; mirrors `Chip::explain_key`. */
export function chipExplainKey(chip: ChipSlug): string {
  return `chip.${chip}`;
}

/** Translation key for a signal's explanation; mirrors `SignalKind::explain_key`. */
export function signalExplainKey(kind: SignalKind): string {
  if (kind.type === "sub_ghz") {
    if (kind.rolling_code === true) return "signal.subghz.rolling";
    if (kind.rolling_code === false) return "signal.subghz.fixed";
    return "signal.subghz.unknown";
  }
  const byType: Record<Exclude<SignalKind["type"], "sub_ghz">, string> = {
    nfc: "signal.nfc",
    lf_rfid: "signal.lfrfid",
    i_button: "signal.ibutton",
    infrared: "signal.infrared",
    bluetooth: "signal.bluetooth",
    gpio: "signal.gpio",
    module: "signal.module",
  };
  return byType[kind.type];
}

/**
 * A one-line summary of a signal, for the collapsed card.
 *
 * Deliberately shows the identifying detail -- the frequency, the UID -- rather
 * than a generic label, so a list of five signals is distinguishable at a glance.
 */
export function signalSummary(kind: SignalKind): string {
  switch (kind.type) {
    case "sub_ghz":
      return `${(kind.frequency_hz / 1_000_000).toFixed(2)} MHz${
        kind.protocol ? ` · ${kind.protocol}` : ""
      }`;
    case "nfc":
      return `${kind.technology} · ${kind.uid}`;
    case "lf_rfid":
      return `${kind.protocol} · ${kind.data}`;
    case "i_button":
      return `${kind.protocol} · ${kind.id}`;
    case "infrared":
      return [kind.protocol, kind.address, kind.command].filter(Boolean).join(" · ");
    case "bluetooth":
      return kind.name ? `${kind.name} · ${kind.address}` : kind.address;
    case "gpio":
      return `${kind.pin} · ${kind.level ? "HIGH" : "LOW"}`;
    case "module":
      return kind.detail ? `${kind.module} · ${kind.detail}` : kind.module;
  }
}

/**
 * Which actions a signal offers. Mirrors `Signal::available_actions`.
 *
 * Rolling-code remotes are deliberately not offered a replay: the code changes
 * on every press, so the button would do nothing and would teach the user the
 * wrong model of how their gate works.
 */
export function availableActions(kind: SignalKind): ActionId[] {
  const actions: ActionId[] = ["save", "analyze", "compare", "export"];

  switch (kind.type) {
    case "sub_ghz":
      if (kind.rolling_code !== true) actions.push("replay");
      break;
    case "infrared":
      actions.push("replay");
      break;
    case "nfc":
    case "i_button":
    case "lf_rfid":
      actions.push("emulate");
      break;
    // Observation only; nothing to send back.
    case "bluetooth":
    case "gpio":
    case "module":
      break;
  }

  return actions;
}
