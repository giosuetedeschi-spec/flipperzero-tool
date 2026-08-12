import type { it } from "./it";

/**
 * English copy.
 *
 * Typed against the Italian catalogue so a key added to one and forgotten in the
 * other fails at compile time rather than showing a raw key to a user.
 */
export const en: Record<keyof typeof it, string> = {
  "app.name": "FlipperZero Tool",

  "nav.radar": "Radar",
  "nav.files": "Files",
  "nav.library": "Library",
  "nav.device": "Device",
  "nav.settings": "Settings",
  "nav.tools": "Tools",
  "nav.sections": "Sections",
  "files.mode.local": "Local",
  "files.mode.serial": "Flipper",
  "files.up": "Up",
  "files.no_folder": "(no folder selected)",
  "files.search": "Search...",
  "files.new": "+ New",
  "files.loading": "Loading...",
  "files.empty": "Empty directory",
  "files.no_match": "No files match your search",
  "files.connect_first": "Connect your Flipper to browse its files",
  "device.disconnected": "Disconnected",
  "tools.analyze": "Raw data analysis",
  "tools.compare": "Compare captures",
  "tools.compare.left": "Left file",
  "tools.compare.right": "Right file",
  "tools.compare.needs_two": "Open at least two files in the editor to compare them.",
  "tools.build": "App development (.fap)",
  "tools.build.desktop_only": "Building .fap files needs uFBT, which only runs on desktop. This section stays visible for clarity, but it cannot be used here.",
  "device.subtitle": "Choose how to reach your Flipper.",
  "settings.language": "Language",
  "settings.mock": "Simulated device",
  "settings.mock.on": "Simulation on",
  "settings.mock.off": "Simulation off",
  "settings.mock.hint": "Shows example signals with no Flipper attached, useful for exploring the app.",
  "settings.geotagging": "Sighting location",
  "settings.geotagging.on": "Geotagging on",
  "settings.geotagging.off": "Geotagging off",
  "settings.geotagging.hint": "Records where you saw each signal. Worth knowing: the history becomes a map of the places you go. It never leaves your phone.",
  "settings.geotagging.denied": "Location permission denied. You can grant it in your phone settings.",
  "settings.geotagging.no_fix": "No GPS fix right now. Indoors this is common -- sightings are saved without a position.",
  "settings.geotagging.unsupported": "This device does not offer geolocation.",
  "settings.geotagging.disabled": "Off.",

  "radar.title": "What the Flipper can see",
  "radar.subtitle": "Tap a chip to see what it is picking up",
  "onboarding.what.title": "This is what your Flipper can hear",
  "onboarding.what.body": "The Flipper has several radios, each sensitive to a different kind of signal: gate remotes, contactless cards, badges, TV remotes. The diagram shows them all -- tap whichever one interests you.",
  "onboarding.counts.title": "The numbers count distinct things",
  "onboarding.counts.body": "The number on a chip is how many *different* signals it has seen, not how many times. Walk past the same gate ten times and it stays one signal, with its sighting count going up.",
  "onboarding.limits.title": "What you can do, and what you cannot",
  "onboarding.limits.body": "Watching and analysing is always fine. Re-transmitting or emulating a signal is not the same thing: only do it on devices you own or have explicit permission to test. In most countries, doing it to someone else's equipment is illegal.",
  "onboarding.next": "Next",
  "onboarding.skip": "Skip",
  "onboarding.start": "Get started",
  "radar.empty": "No signals detected yet",
  "radar.empty.hint":
    "Bring the Flipper near a remote, a card or a reader. Signals show up here as soon as they are picked up.",
  "radar.disconnected": "Flipper not connected",
  "radar.disconnected.hint": "Connect your Flipper to start seeing the signals around you.",
  "radar.signals.count_one": "{count} signal",
  "radar.signals.count_other": "{count} signals",
  "radar.chip.unavailable": "Unavailable in this mode",
  "radar.chip.unavailable.ble":
    "The Flipper's Bluetooth radio is busy talking to this phone, so it cannot also scan for other Bluetooth devices around you. Connect the Flipper over USB to use this.",
  "radar.chip.not_detected": "Module not attached",
  "radar.chip.not_detected.hint":
    "This module plugs into the Flipper's GPIO pins. It appears here as soon as it is detected.",

  "signal.seen_count_one": "Seen {count} time",
  "signal.seen_count_other": "Seen {count} times",
  "signal.first_seen": "First seen",
  "signal.last_seen": "Last seen",
  "signal.strength": "Signal strength",
  "signal.strength.hint": "How strongly it arrives. The closer to zero, the nearer the source.",
  "signal.raw": "Raw data",

  "chip.subghz": "Long-range radio",
  "chip.subghz.detail":
    "Listens to the low-frequency radios used by gate and garage remotes, doorbells, temperature sensors and weather stations. Outdoors, this is the chip that picks up the most.",
  "chip.nfc": "Tap-to-read cards",
  "chip.nfc.detail":
    "Reads the cards you tap on a reader: office badges, transit passes, contactless cards. It only works within a few centimetres.",
  "chip.lfrfid": "Low-frequency badges",
  "chip.lfrfid.detail":
    "Reads older, simpler badges, typical of building entrances and company gates. They usually hold nothing but an ID number.",
  "chip.ibutton": "Metal contact keys",
  "chip.ibutton.detail":
    "The round metal keys you touch against a reader, common on intercoms. They have to physically touch to be read.",
  "chip.infrared": "Infrared remotes",
  "chip.infrared.detail":
    "The invisible light used by TV, air conditioner and set-top box remotes. You have to point the Flipper at the device with nothing in the way.",
  "chip.bluetooth": "Nearby Bluetooth devices",
  "chip.bluetooth.detail":
    "Headphones, watches, trackers and speakers announcing themselves nearby.",
  "chip.gpio": "Connection pins",
  "chip.gpio.detail":
    "The contacts on the Flipper's back where add-on modules and electronic circuits plug in.",
  "chip.wifi_devboard": "WiFi module",
  "chip.wifi_devboard.detail":
    "An ESP32-based add-on board that gives the Flipper WiFi features it does not have on its own.",
  "chip.external_cc1101": "External radio",
  "chip.external_cc1101.detail":
    "A second radio antenna attached externally, with more range than the built-in one.",
  "chip.nrf24": "NRF24 module",
  "chip.nrf24.detail": "A 2.4 GHz radio module, used by some wireless keyboards and mice.",
  "chip.unknown_module": "Unknown module",
  "chip.unknown_module.detail":
    "Something is attached to the GPIO pins but we could not identify it.",

  "signal.subghz.rolling": "Rolling-code remote",
  "signal.subghz.rolling.detail":
    "This remote changes its code on every press, specifically so it cannot be copied. You can see and analyse it, but re-transmitting it would open nothing: the receiver has already discarded that code.",
  "signal.subghz.fixed": "Fixed-code remote",
  "signal.subghz.fixed.detail":
    "Sends the same code on every press. It is the simplest and oldest kind, used by many gates and garages.",
  "signal.subghz.unknown": "Radio signal",
  "signal.subghz.unknown.detail":
    "We picked up a transmission but have not worked out what kind of device it came from.",
  "signal.nfc": "Contactless card",
  "signal.nfc.detail":
    "Every card has a unique serial number, called a UID. Some also hold protected data that cannot always be read.",
  "signal.lfrfid": "125 kHz badge",
  "signal.lfrfid.detail": "A simple badge holding a fixed identification number.",
  "signal.ibutton": "iButton key",
  "signal.ibutton.detail": "A metal key with a unique serial number burned into it.",
  "signal.infrared": "Infrared command",
  "signal.infrared.detail": "A single button from a remote, such as power or channel change.",
  "signal.bluetooth": "Bluetooth device",
  "signal.bluetooth.detail":
    "A device announcing its presence. It does not mean it is connected to you.",
  "signal.gpio": "Pin state",
  "signal.gpio.detail": "The electrical level read on one of the Flipper's contacts.",
  "signal.module": "Attached module",
  "signal.module.detail": "An add-on board that announced itself on the GPIO pins.",

  "action.save": "Save",
  "action.save.detail": "Keep this capture on the Flipper's SD card.",
  "action.replay": "Replay",
  "action.replay.detail": "The Flipper sends this exact signal back out over the air.",
  "action.emulate": "Emulate",
  "action.emulate.detail": "The Flipper pretends to be this card or key in front of a reader.",
  "action.analyze": "Analyse",
  "action.analyze.detail": "Examine the raw data to work out how the signal is built.",
  "action.compare": "Compare",
  "action.compare.detail": "Put this capture next to another one to see what differs.",
  "action.export": "Export",
  "action.export.detail": "Save the capture to your phone or share it.",
  "firmware.official": "Official firmware",
  "firmware.momentum": "Momentum (custom firmware)",
  "firmware.unleashed": "Unleashed (custom firmware)",
  "firmware.roguemaster": "RogueMaster (custom firmware)",
  "firmware.unknown": "Unrecognised firmware",
  "action.needs_fap": "The Signal Radar app must be on the Flipper",
  "action.tx_not_implemented": "The on-device app cannot transmit yet",
  "action.frequency_blocked": "This firmware will not transmit on this frequency",
  "action.compare.pick_second": "Choose a second capture to compare against",

  "legal.title": "Before transmitting",
  "legal.body":
    "You are about to make the Flipper transmit. Only do this on devices you own, or have explicit permission to test. Transmitting at someone else's equipment is illegal in most countries, even out of curiosity.",
  "legal.confirm": "I confirm it is mine or I have permission",
  "legal.cancel": "Cancel",

  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.retry": "Retry",
  "common.unknown": "Unknown",
  "common.never": "Never",
};
