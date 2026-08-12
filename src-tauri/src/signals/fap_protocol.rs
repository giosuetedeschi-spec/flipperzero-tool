//! Decoder for the Signal Radar FAP's telemetry protocol.
//!
//! Mirrors `flipper-fap/signal_radar/protocol.h` field for field. Those tests
//! are the only automated check that the two halves agree: the C side cannot be
//! compiled in CI, because building a FAP needs the Flipper SDK that uFBT
//! fetches from a host the build environment cannot reach.
//!
//! Everything is little-endian, matching the Flipper's ARM core so the firmware
//! side needs no byte swapping.

use super::model::{Chip, Signal, SignalKind};
use crate::errors::AppError;

/// Protocol version this build speaks. Must match `SIGNAL_RADAR_PROTOCOL_VERSION`.
pub const PROTOCOL_VERSION: u8 = 1;

/// version + type + u16 length.
pub const HEADER_SIZE: usize = 4;

/// Largest payload a single event may carry.
pub const MAX_PAYLOAD: usize = 240;

/// RSSI sentinel for chips with no notion of signal strength.
pub const RSSI_UNAVAILABLE: i16 = i16::MIN;

// Commands, phone -> FAP.
pub const CMD_HELLO: u8 = 0x01;
pub const CMD_START_SCAN: u8 = 0x02;
pub const CMD_STOP_SCAN: u8 = 0x03;
pub const CMD_PROBE_MODULES: u8 = 0x04;

// Events, FAP -> phone.
pub const EVT_HELLO: u8 = 0x81;
pub const EVT_SIGNAL: u8 = 0x82;
pub const EVT_CHIP_STATUS: u8 = 0x83;
pub const EVT_ERROR: u8 = 0x84;

// Flags.
pub const FLAG_ROLLING_CODE: u8 = 1 << 0;
pub const FLAG_DECODED: u8 = 1 << 1;
pub const FLAG_SAVED: u8 = 1 << 2;

/// Chip bit positions, as used by the scan mask and the signal event.
pub fn chip_from_wire(value: u8) -> Option<Chip> {
    Some(match value {
        0 => Chip::SubGhz,
        1 => Chip::Nfc,
        2 => Chip::LfRfid,
        3 => Chip::IButton,
        4 => Chip::Infrared,
        5 => Chip::Bluetooth,
        6 => Chip::Gpio,
        7 => Chip::UnknownModule,
        _ => return None,
    })
}

fn chip_to_wire(chip: Chip) -> u8 {
    match chip {
        Chip::SubGhz => 0,
        Chip::Nfc => 1,
        Chip::LfRfid => 2,
        Chip::IButton => 3,
        Chip::Infrared => 4,
        Chip::Bluetooth => 5,
        Chip::Gpio => 6,
        _ => 7,
    }
}

/// State of one chip, from `EVT_CHIP_STATUS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipState {
    Idle,
    Scanning,
    /// Cannot be used right now -- notably Bluetooth while the phone owns the
    /// Flipper's radio.
    Unavailable,
}

/// A decoded message from the FAP.
#[derive(Debug, Clone, PartialEq)]
pub enum FapEvent {
    Hello {
        protocol_version: u8,
        app_major: u8,
        app_minor: u8,
    },
    Signal(Box<DetectedSignal>),
    ChipStatus {
        chip: Chip,
        state: ChipState,
    },
    Error(String),
}

/// The payload of a signal event, before it becomes a [`Signal`].
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedSignal {
    pub chip: Chip,
    pub rssi_dbm: Option<i16>,
    pub frequency_hz: u32,
    pub rolling_code: bool,
    pub decoded: bool,
    pub saved: bool,
    /// Chip-specific identifying bytes: a UID, a key, a decoded frame.
    pub data: Vec<u8>,
    /// Protocol name, e.g. "Princeton", "EM4100".
    pub label: String,
}

impl DetectedSignal {
    /// Promote a detection into the app's unified [`Signal`].
    ///
    /// `seen_at` is supplied by the caller rather than read from the device: the
    /// Flipper has no reliable wall clock, and a signal timestamped from a
    /// device whose clock was never set would sort nonsensically in the history.
    pub fn into_signal(self, seen_at: i64) -> Signal {
        let label = if self.label.is_empty() {
            None
        } else {
            Some(self.label.clone())
        };
        let hex = || {
            self.data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(":")
        };

        let kind = match self.chip {
            Chip::SubGhz => SignalKind::SubGhz {
                frequency_hz: self.frequency_hz,
                preset: None,
                protocol: label,
                // Only claim to know when the frame was actually decoded; an
                // undecoded burst says nothing about its code scheme, and
                // guessing "fixed" would wrongly offer a replay.
                rolling_code: if self.decoded {
                    Some(self.rolling_code)
                } else {
                    None
                },
            },
            Chip::Nfc => SignalKind::Nfc {
                technology: label.unwrap_or_else(|| "Unknown".to_string()),
                uid: hex(),
                atqa: None,
                sak: None,
            },
            Chip::LfRfid => SignalKind::LfRfid {
                protocol: label.unwrap_or_else(|| "Unknown".to_string()),
                data: hex(),
            },
            Chip::IButton => SignalKind::IButton {
                protocol: label.unwrap_or_else(|| "Unknown".to_string()),
                id: hex(),
            },
            Chip::Infrared => SignalKind::Infrared {
                protocol: label,
                address: None,
                command: Some(hex()),
            },
            Chip::Bluetooth => SignalKind::Bluetooth {
                address: hex(),
                name: label,
            },
            Chip::Gpio => SignalKind::Gpio {
                pin: label.unwrap_or_else(|| "?".to_string()),
                level: self.data.first().copied().unwrap_or(0) != 0,
            },
            _ => SignalKind::Module {
                module: label.unwrap_or_else(|| "Unknown".to_string()),
                detail: None,
            },
        };

        let mut signal = Signal::new(kind, seen_at).with_raw(self.data);
        if let Some(rssi) = self.rssi_dbm {
            signal = signal.with_rssi(rssi);
        }
        signal
    }
}

/// Build a command to send to the FAP.
pub fn encode_command(command: u8, payload: &[u8]) -> Result<Vec<u8>, AppError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(AppError::ParseError(format!(
            "Payload of {} bytes exceeds the {} byte limit",
            payload.len(),
            MAX_PAYLOAD
        )));
    }
    let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
    out.push(PROTOCOL_VERSION);
    out.push(command);
    out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

/// Build the chip mask for `CMD_START_SCAN`.
///
/// A mask rather than a list so the phone can ask for exactly the chips whose
/// screen is open; scanning every radio drains the battery for data nobody is
/// looking at.
pub fn encode_scan_mask(chips: &[Chip]) -> u8 {
    chips
        .iter()
        .fold(0u8, |mask, chip| mask | (1 << chip_to_wire(*chip)))
}

/// Read a little-endian `u16` at `offset`.
fn read_u16(data: &[u8], offset: usize) -> Result<u16, AppError> {
    data.get(offset..offset + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| AppError::ParseError("Message truncated".to_string()))
}

/// Decode one message from the FAP.
///
/// Returns the event and how many bytes it consumed, so a caller can walk a
/// buffer holding several. `Ok(None)` means the message has not fully arrived
/// yet -- the normal case mid-transfer, not an error.
pub fn decode_event(data: &[u8]) -> Result<Option<(FapEvent, usize)>, AppError> {
    if data.len() < HEADER_SIZE {
        return Ok(None);
    }

    let version = data[0];
    let event_type = data[1];
    let length = read_u16(data, 2)? as usize;

    if length > MAX_PAYLOAD {
        return Err(AppError::ParseError(format!(
            "Event claims {} payload bytes, over the {} byte limit; the stream is desynchronised",
            length, MAX_PAYLOAD
        )));
    }

    let total = HEADER_SIZE + length;
    if data.len() < total {
        return Ok(None);
    }

    // Check the version only once the message is whole, so a version mismatch
    // is reported against a real message rather than a partial read. Hello is
    // exempt: it is precisely how the phone discovers the mismatch.
    if version != PROTOCOL_VERSION && event_type != EVT_HELLO {
        return Err(AppError::ParseError(format!(
            "FAP speaks protocol v{}, this build expects v{}; redeploy the app",
            version, PROTOCOL_VERSION
        )));
    }

    let payload = &data[HEADER_SIZE..total];

    let event = match event_type {
        EVT_HELLO => {
            if payload.len() < 3 {
                return Err(AppError::ParseError("Hello payload too short".to_string()));
            }
            FapEvent::Hello {
                protocol_version: payload[0],
                app_major: payload[1],
                app_minor: payload[2],
            }
        }
        EVT_SIGNAL => FapEvent::Signal(Box::new(decode_signal_payload(payload)?)),
        EVT_CHIP_STATUS => {
            if payload.len() < 2 {
                return Err(AppError::ParseError(
                    "Chip status payload too short".to_string(),
                ));
            }
            let chip = chip_from_wire(payload[0])
                .ok_or_else(|| AppError::ParseError(format!("Unknown chip {}", payload[0])))?;
            let state = match payload[1] {
                0 => ChipState::Idle,
                1 => ChipState::Scanning,
                2 => ChipState::Unavailable,
                other => {
                    return Err(AppError::ParseError(format!(
                        "Unknown chip state {}",
                        other
                    )));
                }
            };
            FapEvent::ChipStatus { chip, state }
        }
        EVT_ERROR => FapEvent::Error(String::from_utf8_lossy(payload).into_owned()),
        other => {
            return Err(AppError::ParseError(format!(
                "Unknown event type 0x{:02X}",
                other
            )));
        }
    };

    Ok(Some((event, total)))
}

fn decode_signal_payload(payload: &[u8]) -> Result<DetectedSignal, AppError> {
    // chip(1) + rssi(2) + frequency(4) + flags(1) + data_len(1)
    const FIXED: usize = 9;
    if payload.len() < FIXED {
        return Err(AppError::ParseError("Signal payload too short".to_string()));
    }

    let chip = chip_from_wire(payload[0])
        .ok_or_else(|| AppError::ParseError(format!("Unknown chip {}", payload[0])))?;
    let rssi_raw = i16::from_le_bytes([payload[1], payload[2]]);
    let frequency_hz = u32::from_le_bytes([payload[3], payload[4], payload[5], payload[6]]);
    let flags = payload[7];
    let data_len = payload[8] as usize;

    let data_end = FIXED + data_len;
    let data = payload
        .get(FIXED..data_end)
        .ok_or_else(|| AppError::ParseError("Signal data runs past the payload".to_string()))?
        .to_vec();

    let label_len = *payload
        .get(data_end)
        .ok_or_else(|| AppError::ParseError("Signal label length missing".to_string()))?
        as usize;
    let label_bytes = payload
        .get(data_end + 1..data_end + 1 + label_len)
        .ok_or_else(|| AppError::ParseError("Signal label runs past the payload".to_string()))?;

    Ok(DetectedSignal {
        chip,
        // The sentinel means "this chip has no notion of strength", which is
        // different from "strength is very low" and must not become -32768 dBm
        // in the UI.
        rssi_dbm: if rssi_raw == RSSI_UNAVAILABLE {
            None
        } else {
            Some(rssi_raw)
        },
        frequency_hz,
        rolling_code: flags & FLAG_ROLLING_CODE != 0,
        decoded: flags & FLAG_DECODED != 0,
        saved: flags & FLAG_SAVED != 0,
        data,
        label: String::from_utf8_lossy(label_bytes).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a signal event the way the C side lays it out.
    fn signal_event(
        chip: u8,
        rssi: i16,
        frequency: u32,
        flags: u8,
        data: &[u8],
        label: &str,
    ) -> Vec<u8> {
        let mut payload = vec![chip];
        payload.extend_from_slice(&rssi.to_le_bytes());
        payload.extend_from_slice(&frequency.to_le_bytes());
        payload.push(flags);
        payload.push(data.len() as u8);
        payload.extend_from_slice(data);
        payload.push(label.len() as u8);
        payload.extend_from_slice(label.as_bytes());

        let mut out = vec![PROTOCOL_VERSION, EVT_SIGNAL];
        out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&payload);
        out
    }

    #[test]
    fn commands_carry_version_type_and_length() {
        let encoded = encode_command(CMD_START_SCAN, &[0b0000_0011]).unwrap();
        assert_eq!(encoded[0], PROTOCOL_VERSION);
        assert_eq!(encoded[1], CMD_START_SCAN);
        assert_eq!(u16::from_le_bytes([encoded[2], encoded[3]]), 1);
        assert_eq!(encoded[4], 0b0000_0011);
    }

    #[test]
    fn an_oversized_command_is_refused_rather_than_truncated() {
        assert!(encode_command(CMD_HELLO, &vec![0u8; MAX_PAYLOAD + 1]).is_err());
    }

    #[test]
    fn the_scan_mask_selects_only_the_requested_chips() {
        let mask = encode_scan_mask(&[Chip::SubGhz, Chip::Nfc]);
        assert_eq!(mask, 0b0000_0011);
        assert_eq!(encode_scan_mask(&[]), 0);
        assert_eq!(encode_scan_mask(&[Chip::Gpio]), 0b0100_0000);
    }

    #[test]
    fn a_partial_message_is_not_an_error() {
        // Mid-transfer over BLE this is the normal state of the buffer.
        assert!(decode_event(&[]).unwrap().is_none());
        assert!(
            decode_event(&[PROTOCOL_VERSION, EVT_HELLO])
                .unwrap()
                .is_none()
        );

        let full = signal_event(0, -50, 433_920_000, FLAG_DECODED, &[0xAA], "Princeton");
        for cut in 0..full.len() {
            assert!(
                decode_event(&full[..cut]).unwrap().is_none(),
                "{} of {} bytes should not decode",
                cut,
                full.len()
            );
        }
        assert!(decode_event(&full).unwrap().is_some());
    }

    #[test]
    fn decoding_reports_how_many_bytes_it_consumed() {
        let mut stream = signal_event(
            1,
            RSSI_UNAVAILABLE,
            0,
            FLAG_DECODED,
            &[0x04, 0x1E],
            "ISO14443",
        );
        let first_len = stream.len();
        stream.extend_from_slice(&signal_event(0, -60, 433_920_000, 0, &[0x01], "RAW"));

        let (_, consumed) = decode_event(&stream).unwrap().unwrap();
        assert_eq!(consumed, first_len);

        let (second, _) = decode_event(&stream[consumed..]).unwrap().unwrap();
        assert!(matches!(second, FapEvent::Signal(_)));
    }

    #[test]
    fn a_signal_event_round_trips_its_fields() {
        let raw = signal_event(
            0,
            -63,
            433_920_000,
            FLAG_DECODED | FLAG_ROLLING_CODE,
            &[0xDE, 0xAD],
            "KeeLoq",
        );
        let (event, _) = decode_event(&raw).unwrap().unwrap();

        let FapEvent::Signal(detected) = event else {
            panic!("expected a signal event");
        };
        assert_eq!(detected.chip, Chip::SubGhz);
        assert_eq!(detected.rssi_dbm, Some(-63));
        assert_eq!(detected.frequency_hz, 433_920_000);
        assert!(detected.rolling_code);
        assert!(detected.decoded);
        assert!(!detected.saved);
        assert_eq!(detected.data, vec![0xDE, 0xAD]);
        assert_eq!(detected.label, "KeeLoq");
    }

    #[test]
    fn the_rssi_sentinel_becomes_absent_not_a_huge_negative_reading() {
        // -32768 dBm rendered in a strength bar would be absurd; chips without a
        // notion of strength must report nothing at all.
        let raw = signal_event(1, RSSI_UNAVAILABLE, 0, FLAG_DECODED, &[0x01], "NFC");
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        let FapEvent::Signal(detected) = event else {
            panic!("expected a signal event");
        };
        assert_eq!(detected.rssi_dbm, None);
    }

    #[test]
    fn an_undecoded_burst_does_not_claim_to_know_its_code_scheme() {
        // Guessing "fixed" here would offer a Replay button on what might be a
        // rolling-code remote.
        let raw = signal_event(0, -70, 433_920_000, 0, &[0x01], "");
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        let FapEvent::Signal(detected) = event else {
            panic!("expected a signal event");
        };

        let signal = detected.into_signal(1_000);
        match signal.kind {
            SignalKind::SubGhz { rolling_code, .. } => assert_eq!(rolling_code, None),
            other => panic!("expected Sub-GHz, got {:?}", other),
        }
    }

    #[test]
    fn a_decoded_rolling_code_remote_is_marked_as_one() {
        let raw = signal_event(
            0,
            -55,
            868_350_000,
            FLAG_DECODED | FLAG_ROLLING_CODE,
            &[0x09],
            "KeeLoq",
        );
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        let FapEvent::Signal(detected) = event else {
            panic!("expected a signal event");
        };

        let signal = detected.into_signal(1_000);
        assert_eq!(signal.explain_key(), "signal.subghz.rolling");
        assert!(
            !signal
                .available_actions()
                .contains(&super::super::model::ActionId::Replay),
            "a rolling code must never offer a replay"
        );
    }

    #[test]
    fn a_detected_tag_becomes_an_nfc_signal_with_a_readable_uid() {
        let raw = signal_event(
            1,
            RSSI_UNAVAILABLE,
            0,
            FLAG_DECODED,
            &[0x04, 0x1E, 0x23, 0x4A],
            "ISO14443-3A",
        );
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        let FapEvent::Signal(detected) = event else {
            panic!("expected a signal event");
        };

        let signal = detected.into_signal(1_000);
        assert_eq!(signal.chip, Chip::Nfc);
        match signal.kind {
            SignalKind::Nfc {
                uid, technology, ..
            } => {
                assert_eq!(uid, "04:1E:23:4A");
                assert_eq!(technology, "ISO14443-3A");
            }
            other => panic!("expected NFC, got {:?}", other),
        }
    }

    #[test]
    fn the_same_detection_twice_yields_the_same_id() {
        // The fingerprint is what turns repeat sightings into a count.
        let raw = signal_event(0, -60, 433_920_000, FLAG_DECODED, &[0x01], "Princeton");
        let (a, _) = decode_event(&raw).unwrap().unwrap();
        let (b, _) = decode_event(&raw).unwrap().unwrap();

        let (FapEvent::Signal(a), FapEvent::Signal(b)) = (a, b) else {
            panic!("expected signal events");
        };
        assert_eq!(a.into_signal(10).id, b.into_signal(9_999).id);
    }

    #[test]
    fn hello_decodes_even_when_the_version_disagrees() {
        // Hello is how the phone *discovers* a mismatch, so it must not be
        // rejected by the very check it exists to trigger.
        let raw = vec![99, EVT_HELLO, 3, 0, 7, 2, 5];
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        assert_eq!(
            event,
            FapEvent::Hello {
                protocol_version: 7,
                app_major: 2,
                app_minor: 5,
            }
        );
    }

    #[test]
    fn any_other_event_from_a_mismatched_version_is_refused() {
        let mut raw = signal_event(0, -60, 433_920_000, FLAG_DECODED, &[0x01], "P");
        raw[0] = PROTOCOL_VERSION + 1;
        let err = decode_event(&raw).unwrap_err();
        assert!(format!("{}", err).contains("redeploy"), "got {}", err);
    }

    #[test]
    fn chip_status_carries_the_unavailable_case() {
        // The reason this state exists: BLE cannot scan while the phone owns
        // the radio, and the UI must say so rather than show an empty list.
        let raw = vec![PROTOCOL_VERSION, EVT_CHIP_STATUS, 2, 0, 5, 2];
        let (event, _) = decode_event(&raw).unwrap().unwrap();
        assert_eq!(
            event,
            FapEvent::ChipStatus {
                chip: Chip::Bluetooth,
                state: ChipState::Unavailable,
            }
        );
    }

    #[test]
    fn errors_arrive_as_readable_text() {
        let message = b"CC1101 not responding";
        let mut raw = vec![PROTOCOL_VERSION, EVT_ERROR];
        raw.extend_from_slice(&(message.len() as u16).to_le_bytes());
        raw.extend_from_slice(message);

        let (event, _) = decode_event(&raw).unwrap().unwrap();
        assert_eq!(event, FapEvent::Error("CC1101 not responding".to_string()));
    }

    #[test]
    fn an_absurd_length_is_rejected_rather_than_allocated() {
        let raw = vec![PROTOCOL_VERSION, EVT_SIGNAL, 0xFF, 0xFF];
        assert!(decode_event(&raw).is_err());
    }

    #[test]
    fn an_unknown_event_type_is_reported() {
        let raw = vec![PROTOCOL_VERSION, 0xEE, 0, 0];
        assert!(decode_event(&raw).is_err());
    }

    #[test]
    fn a_label_running_past_the_payload_is_rejected() {
        // A corrupt length must not read adjacent memory or panic.
        let mut raw = signal_event(0, -60, 433_920_000, FLAG_DECODED, &[0x01], "AB");
        *raw.last_mut().unwrap() = b'A';
        let payload_start = HEADER_SIZE;
        // Claim a 200-byte label in a payload that has two bytes left.
        raw[payload_start + 9 + 1] = 200;
        assert!(decode_event(&raw).is_err());
    }

    #[test]
    fn every_chip_maps_both_ways() {
        for chip in Chip::all() {
            let wire = chip_to_wire(*chip);
            let back = chip_from_wire(wire).expect("wire value must map back");
            // External modules all share the one wire value, by design.
            if chip.is_external_module() {
                assert!(back.is_external_module());
            } else {
                assert_eq!(back, *chip);
            }
        }
    }
}
