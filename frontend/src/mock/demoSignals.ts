import type { Signal } from "../types/signals";

/**
 * A believable set of signals for demo and mock mode.
 *
 * Chosen to cover the cases the Radar has to get right rather than to look
 * impressive: a fixed-code remote next to a rolling-code one (which must not
 * offer a replay), a signal seen many times versus one seen once, readings with
 * no RSSI at all, and one geotagged sighting.
 *
 * Timestamps are relative to load, so the demo never looks stale.
 */
const NOW = Math.floor(Date.now() / 1000);

export const DEMO_SIGNALS: Signal[] = [
  {
    id: "subghz-1a2b3c4d5e6f7081",
    chip: "subghz",
    kind: {
      type: "sub_ghz",
      frequency_hz: 433_920_000,
      preset: "AM650",
      protocol: "Princeton",
      rolling_code: false,
    },
    first_seen: NOW - 86_400 * 3,
    last_seen: NOW - 120,
    count: 27,
    rssi_dbm: -54,
    location: { latitude: 45.0703, longitude: 7.6869 },
  },
  {
    id: "subghz-90a1b2c3d4e5f607",
    chip: "subghz",
    kind: {
      type: "sub_ghz",
      frequency_hz: 868_350_000,
      preset: "FM238",
      protocol: "KeeLoq",
      rolling_code: true,
    },
    first_seen: NOW - 3_600,
    last_seen: NOW - 45,
    count: 4,
    rssi_dbm: -71,
    location: null,
  },
  {
    id: "nfc-aabbccddeeff0011",
    chip: "nfc",
    kind: {
      type: "nfc",
      technology: "ISO14443-3A",
      uid: "04:1E:23:4A:5B:6C",
      atqa: "00 44",
      sak: "08",
    },
    first_seen: NOW - 86_400,
    last_seen: NOW - 600,
    count: 3,
    rssi_dbm: null,
    location: null,
  },
  {
    id: "ibutton-1122334455667788",
    chip: "ibutton",
    kind: { type: "i_button", protocol: "Dallas", id: "01:02:03:04:05:06:07:08" },
    first_seen: NOW - 86_400 * 10,
    last_seen: NOW - 86_400 * 2,
    count: 1,
    rssi_dbm: null,
    location: null,
  },
  {
    id: "lfrfid-99aabbccddeeff00",
    chip: "lfrfid",
    kind: { type: "lf_rfid", protocol: "EM4100", data: "12 34 56 78 90" },
    first_seen: NOW - 7_200,
    last_seen: NOW - 300,
    count: 6,
    rssi_dbm: null,
    location: null,
  },
  {
    id: "infrared-5566778899aabbcc",
    chip: "infrared",
    kind: { type: "infrared", protocol: "NEC", address: "04 00 00 00", command: "08 00 00 00" },
    first_seen: NOW - 1_800,
    last_seen: NOW - 90,
    count: 12,
    rssi_dbm: null,
    location: null,
  },
];
