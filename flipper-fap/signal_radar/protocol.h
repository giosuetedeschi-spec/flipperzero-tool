/**
 * Wire protocol between the Signal Radar FAP and the phone.
 *
 * This header is the single source of truth for the format. The Rust decoder in
 * `src-tauri/src/signals/fap_protocol.rs` mirrors it field for field, and its
 * tests are the only automated check that the two agree -- the C side cannot be
 * compiled in CI, because building a FAP needs the Flipper SDK, which uFBT
 * fetches from a host the build environment cannot reach.
 *
 * Design constraints that shaped this:
 *
 *   - BLE carries a few kilobytes per second. JSON would spend most of that on
 *     punctuation, so events are packed binary.
 *   - Every event is self-delimiting and carries its own length, so a truncated
 *     transfer costs one event rather than desynchronising the stream.
 *   - The version byte is first in every message. A phone talking to an
 *     outdated FAP must be able to tell immediately and redeploy, rather than
 *     misparsing fields into plausible nonsense.
 *
 * All multi-byte integers are little-endian, matching the Flipper's ARM core so
 * the firmware side needs no byte swapping.
 */

#pragma once

#include <stdbool.h>
#include <stdint.h>

/**
 * Protocol version.
 *
 * Bump on ANY layout change. The phone compares this against its own and
 * redeploys the FAP on mismatch, so a stale app never misreads a new layout.
 */
#define SIGNAL_RADAR_PROTOCOL_VERSION 1

/** Longest payload a single event may carry, after the header. */
#define SIGNAL_RADAR_MAX_PAYLOAD 240

/** Message types, phone -> FAP. */
typedef enum {
    /** Ask for the protocol/app version. Always answered, even on mismatch. */
    SignalRadarCmdHello = 0x01,
    /** Begin scanning the given chips. Payload: one chip-mask byte. */
    SignalRadarCmdStartScan = 0x02,
    /** Stop all scanning. No payload. */
    SignalRadarCmdStopScan = 0x03,
    /** Report which external GPIO modules are attached. No payload. */
    SignalRadarCmdProbeModules = 0x04,
} SignalRadarCommand;

/** Message types, FAP -> phone. */
typedef enum {
    /** Answer to Hello. Payload: protocol version, app major, app minor. */
    SignalRadarEvtHello = 0x81,
    /** One detected signal. Payload: SignalRadarSignalEvent. */
    SignalRadarEvtSignal = 0x82,
    /** A chip changed state (started, stopped, unavailable). */
    SignalRadarEvtChipStatus = 0x83,
    /** Something went wrong; payload is a UTF-8 message. */
    SignalRadarEvtError = 0x84,
} SignalRadarEventType;

/**
 * Chips, as bit positions.
 *
 * A mask rather than a list so StartScan fits in one byte, and so the phone can
 * ask for exactly the chips whose screen is open -- scanning every radio at once
 * drains the battery for data nobody is looking at.
 */
typedef enum {
    SignalRadarChipSubGhz = 0,
    SignalRadarChipNfc = 1,
    SignalRadarChipLfRfid = 2,
    SignalRadarChipIButton = 3,
    SignalRadarChipInfrared = 4,
    SignalRadarChipBluetooth = 5,
    SignalRadarChipGpio = 6,
    SignalRadarChipExternalModule = 7,
} SignalRadarChip;

/** Chip availability, reported by SignalRadarEvtChipStatus. */
typedef enum {
    SignalRadarChipIdle = 0,
    SignalRadarChipScanning = 1,
    /**
     * The chip cannot be used right now, and why.
     *
     * The important case: while the phone is connected over BLE, the Flipper's
     * own radio is busy serving that link and cannot scan for other Bluetooth
     * devices. The app shows this instead of an empty list, which would read as
     * "nothing nearby" -- a lie.
     */
    SignalRadarChipUnavailable = 2,
} SignalRadarChipState;

/**
 * Every message, both directions:
 *
 *   offset 0: uint8  version
 *   offset 1: uint8  type       (SignalRadarCommand or SignalRadarEventType)
 *   offset 2: uint16 length     (payload bytes that follow, little-endian)
 *   offset 4: payload
 */
#define SIGNAL_RADAR_HEADER_SIZE 4

/**
 * Payload of SignalRadarEvtSignal.
 *
 * Laid out with the fixed fields first so a decoder can read them without
 * having walked the variable-length tail.
 *
 *   uint8  chip           (SignalRadarChip)
 *   int16  rssi_dbm       (INT16_MIN when the chip has no notion of strength)
 *   uint32 frequency_hz   (0 when not applicable)
 *   uint8  flags          (see below)
 *   uint8  data_len
 *   uint8  data[data_len] chip-specific identifying bytes: a UID, a key, a
 *                         decoded IR frame
 *   uint8  label_len
 *   char   label[label_len] protocol name, e.g. "Princeton", "EM4100". Not
 *                           null-terminated.
 */
#define SIGNAL_RADAR_RSSI_UNAVAILABLE INT16_MIN

/** Flags in the signal event. */
typedef enum {
    /**
     * The transmitter uses a rolling code.
     *
     * Carried explicitly rather than inferred from the protocol name, because
     * the app hides the Replay button on this basis: replaying a rolling code
     * does nothing, and offering the button teaches the user a wrong model of
     * how their own gate works.
     */
    SignalRadarFlagRollingCode = 1 << 0,
    /** The signal was fully decoded, not just detected as energy. */
    SignalRadarFlagDecoded = 1 << 1,
    /** The capture was saved to the SD card by the FAP. */
    SignalRadarFlagSaved = 1 << 2,
} SignalRadarFlags;
