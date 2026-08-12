//! Performing an action on a detected signal.
//!
//! Until now the Radar offered six buttons and none of them did anything. This
//! is where they become real -- and where the ones that cannot yet be real say
//! so precisely, rather than failing silently or pretending to succeed.
//!
//! Three of the six are answerable entirely on this side: analysis reuses the
//! existing byte-level engine, and saving and exporting are file operations.
//! Replaying and emulating need the device to transmit, which means the Signal
//! Radar FAP: until it is deployed and reports the capability, those return an
//! error naming what is missing.

use super::firmware::FirmwareCapabilities;
use super::model::{ActionId, Signal, SignalKind};
use crate::errors::AppError;
use crate::reverse_engineer::{AnalysisResult, analyze};
use serde::{Deserialize, Serialize};

/// What performing an action produced.
///
/// A single type rather than one per action, so the frontend has one shape to
/// handle and adding an action does not change the IPC surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionOutcome {
    /// Byte-level analysis of the capture.
    Analysis(Box<AnalysisResult>),
    /// The signal was written somewhere; the path is where.
    Saved { path: String },
    /// Text ready for the caller to put on a clipboard or into a file.
    Exported { filename: String, contents: String },
    /// The action needs hardware or a capability that is not present.
    Unavailable { reason_key: String, detail: String },
}

/// Where a capture of this kind belongs on the SD card.
///
/// Matches the firmware's own layout so files saved by the app show up in the
/// Flipper's native menus rather than in a folder only this app knows about.
pub fn sd_card_directory(kind: &SignalKind) -> &'static str {
    match kind {
        SignalKind::SubGhz { .. } => "/ext/subghz",
        SignalKind::Nfc { .. } => "/ext/nfc",
        SignalKind::LfRfid { .. } => "/ext/lfrfid",
        SignalKind::IButton { .. } => "/ext/ibutton",
        SignalKind::Infrared { .. } => "/ext/infrared",
        // No native menu owns these, so they go somewhere clearly ours.
        SignalKind::Bluetooth { .. } | SignalKind::Gpio { .. } | SignalKind::Module { .. } => {
            "/ext/apps_data/signal_radar"
        }
    }
}

/// The extension the Flipper expects for this kind of capture.
pub fn file_extension(kind: &SignalKind) -> &'static str {
    match kind {
        SignalKind::SubGhz { .. } => "sub",
        SignalKind::Nfc { .. } => "nfc",
        SignalKind::LfRfid { .. } => "rfid",
        SignalKind::IButton { .. } => "ibtn",
        SignalKind::Infrared { .. } => "ir",
        SignalKind::Bluetooth { .. } | SignalKind::Gpio { .. } | SignalKind::Module { .. } => "txt",
    }
}

/// Render a signal as the Flipper's own key-value file format.
///
/// Producing the native format rather than something bespoke is what makes a
/// saved capture usable from the device itself, not just from this app.
pub fn render_capture(signal: &Signal) -> String {
    let mut out = String::new();

    match &signal.kind {
        SignalKind::SubGhz {
            frequency_hz,
            preset,
            protocol,
            ..
        } => {
            out.push_str("Filetype: Flipper SubGhz Key File\nVersion: 1\n");
            out.push_str(&format!("Frequency: {}\n", frequency_hz));
            out.push_str(&format!(
                "Preset: {}\n",
                preset
                    .as_deref()
                    .unwrap_or("FuriHalSubGhzPresetOok650Async")
            ));
            out.push_str(&format!(
                "Protocol: {}\n",
                protocol.as_deref().unwrap_or("RAW")
            ));
        }
        SignalKind::Nfc {
            technology,
            uid,
            atqa,
            sak,
        } => {
            out.push_str("Filetype: Flipper NFC device\nVersion: 4\n");
            out.push_str(&format!("Device type: {}\n", technology));
            out.push_str(&format!("UID: {}\n", uid));
            if let Some(atqa) = atqa {
                out.push_str(&format!("ATQA: {}\n", atqa));
            }
            if let Some(sak) = sak {
                out.push_str(&format!("SAK: {}\n", sak));
            }
        }
        SignalKind::LfRfid { protocol, data } => {
            out.push_str("Filetype: Flipper RFID key\nVersion: 1\n");
            out.push_str(&format!("Key type: {}\n", protocol));
            out.push_str(&format!("Data: {}\n", data));
        }
        SignalKind::IButton { protocol, id } => {
            out.push_str("Filetype: Flipper iButton key\nVersion: 1\n");
            out.push_str(&format!("Key type: {}\n", protocol));
            out.push_str(&format!("Data: {}\n", id));
        }
        SignalKind::Infrared {
            protocol,
            address,
            command,
        } => {
            out.push_str("Filetype: IR signals file\nVersion: 1\n");
            out.push_str("name: Captured\ntype: parsed\n");
            out.push_str(&format!(
                "protocol: {}\n",
                protocol.as_deref().unwrap_or("Unknown")
            ));
            out.push_str(&format!(
                "address: {}\n",
                address.as_deref().unwrap_or("00 00 00 00")
            ));
            out.push_str(&format!(
                "command: {}\n",
                command.as_deref().unwrap_or("00 00 00 00")
            ));
        }
        SignalKind::Bluetooth { address, name } => {
            out.push_str(&format!("Address: {}\n", address));
            out.push_str(&format!("Name: {}\n", name.as_deref().unwrap_or("")));
        }
        SignalKind::Gpio { pin, level } => {
            out.push_str(&format!("Pin: {}\nLevel: {}\n", pin, level));
        }
        SignalKind::Module { module, detail } => {
            out.push_str(&format!("Module: {}\n", module));
            out.push_str(&format!("Detail: {}\n", detail.as_deref().unwrap_or("")));
        }
    }

    out
}

/// A filename that identifies the capture without colliding.
///
/// The fingerprint is already unique per signal, so two captures of the same
/// remote overwrite rather than accumulating near-identical files.
pub fn capture_filename(signal: &Signal) -> String {
    format!("{}.{}", signal.id, file_extension(&signal.kind))
}

/// Full destination path on the SD card.
pub fn capture_path(signal: &Signal) -> String {
    format!(
        "{}/{}",
        sd_card_directory(&signal.kind),
        capture_filename(signal)
    )
}

/// Whether the device can currently perform a transmitting action.
///
/// Kept as an explicit input rather than read from a global, so the caller has
/// to decide what it knows about the device instead of assuming.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    /// A Signal Radar FAP is running and reported a compatible version.
    pub fap_available: bool,
    /// What the connected firmware permits.
    pub firmware: FirmwareCapabilities,
}

impl DeviceCapabilities {
    /// Nothing is known about the device yet.
    pub fn none() -> Self {
        Self {
            fap_available: false,
            firmware: FirmwareCapabilities::unknown(),
        }
    }
}

/// Perform an action that needs nothing beyond the signal itself.
///
/// Actions requiring transmission are reported as unavailable with a reason
/// rather than attempted, because a silent no-op on a Replay button is exactly
/// the failure that teaches a user to distrust the app.
pub fn perform(
    signal: &Signal,
    action: ActionId,
    capabilities: &DeviceCapabilities,
) -> Result<ActionOutcome, AppError> {
    // Guard first: an action the signal does not offer should never arrive here,
    // and if it does, something upstream is out of step with the model.
    if !signal.available_actions().contains(&action) {
        return Err(AppError::General(format!(
            "{:?} is not available for this signal",
            action
        )));
    }

    match action {
        ActionId::Analyze => Ok(ActionOutcome::Analysis(Box::new(analyze(&signal.raw)))),

        ActionId::Save => Ok(ActionOutcome::Saved {
            path: capture_path(signal),
        }),

        ActionId::Export => Ok(ActionOutcome::Exported {
            filename: capture_filename(signal),
            contents: render_capture(signal),
        }),

        // Comparison needs a second capture, which only the caller can choose.
        ActionId::Compare => Ok(ActionOutcome::Unavailable {
            reason_key: "action.compare.pick_second".to_string(),
            detail: "Choose another capture to compare against".to_string(),
        }),

        ActionId::Replay | ActionId::Emulate => {
            // Refuse an out-of-band frequency before anything else. Stock
            // firmware silently declines it, which surfaces as a replay that
            // appears to work and changes nothing -- the most confusing failure
            // the app can produce.
            if let SignalKind::SubGhz { frequency_hz, .. } = &signal.kind
                && !capabilities.firmware.can_transmit_on(*frequency_hz)
            {
                return Ok(ActionOutcome::Unavailable {
                    reason_key: "action.frequency_blocked".to_string(),
                    detail: format!(
                        "This firmware will not transmit on {:.2} MHz",
                        *frequency_hz as f64 / 1_000_000.0
                    ),
                });
            }

            if !capabilities.fap_available {
                return Ok(ActionOutcome::Unavailable {
                    reason_key: "action.needs_fap".to_string(),
                    detail: "Transmitting needs the Signal Radar app on the Flipper".to_string(),
                });
            }
            // The FAP protocol carries no transmit command yet; claiming success
            // would be worse than saying so.
            Ok(ActionOutcome::Unavailable {
                reason_key: "action.tx_not_implemented".to_string(),
                detail: "The on-device app cannot transmit yet".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::model::Signal;

    fn subghz(rolling: Option<bool>) -> Signal {
        Signal::new(
            SignalKind::SubGhz {
                frequency_hz: 433_920_000,
                preset: Some("AM650".to_string()),
                protocol: Some("Princeton".to_string()),
                rolling_code: rolling,
            },
            1_000,
        )
        .with_raw(vec![0xDE, 0xAD, 0xBE, 0xEF])
    }

    fn tag() -> Signal {
        Signal::new(
            SignalKind::Nfc {
                technology: "ISO14443-3A".to_string(),
                uid: "04:1E:23".to_string(),
                atqa: Some("00 44".to_string()),
                sak: Some("08".to_string()),
            },
            1_000,
        )
    }

    #[test]
    fn analysis_runs_over_the_captured_bytes() {
        let outcome = perform(
            &subghz(Some(false)),
            ActionId::Analyze,
            &DeviceCapabilities::none(),
        )
        .expect("analysis needs no device");
        match outcome {
            ActionOutcome::Analysis(result) => assert!(result.entropy > 0.0),
            other => panic!("expected an analysis, got {:?}", other),
        }
    }

    #[test]
    fn captures_are_saved_where_the_firmware_looks_for_them() {
        // Saving into a folder only this app knows about would make the capture
        // invisible from the Flipper's own menus.
        let outcome = perform(
            &subghz(Some(false)),
            ActionId::Save,
            &DeviceCapabilities::none(),
        )
        .unwrap();
        match outcome {
            ActionOutcome::Saved { path } => {
                assert!(path.starts_with("/ext/subghz/"), "got {}", path);
                assert!(path.ends_with(".sub"), "got {}", path);
            }
            other => panic!("expected a save, got {:?}", other),
        }
    }

    #[test]
    fn each_kind_lands_in_its_own_directory_with_its_own_extension() {
        assert_eq!(sd_card_directory(&tag().kind), "/ext/nfc");
        assert_eq!(file_extension(&tag().kind), "nfc");

        let ibutton = Signal::new(
            SignalKind::IButton {
                protocol: "Dallas".to_string(),
                id: "01:02".to_string(),
            },
            0,
        );
        assert_eq!(sd_card_directory(&ibutton.kind), "/ext/ibutton");
        assert_eq!(file_extension(&ibutton.kind), "ibtn");
    }

    #[test]
    fn two_captures_of_the_same_remote_share_a_filename() {
        // Identity is the fingerprint, so re-seeing a remote overwrites rather
        // than littering the card with near-identical files.
        let first = subghz(Some(false));
        let second = subghz(Some(false));
        assert_eq!(capture_path(&first), capture_path(&second));
    }

    #[test]
    fn an_exported_capture_is_in_the_flippers_own_format() {
        let outcome = perform(
            &subghz(Some(false)),
            ActionId::Export,
            &DeviceCapabilities::none(),
        )
        .unwrap();
        match outcome {
            ActionOutcome::Exported { contents, filename } => {
                assert!(contents.starts_with("Filetype: Flipper SubGhz Key File"));
                assert!(contents.contains("Frequency: 433920000"));
                assert!(contents.contains("Protocol: Princeton"));
                assert!(filename.ends_with(".sub"));
            }
            other => panic!("expected an export, got {:?}", other),
        }
    }

    #[test]
    fn an_exported_capture_can_be_read_back_by_our_own_parser() {
        // The strongest available check that the rendering is real: round-trip
        // it through the parser the app uses for files off the SD card.
        let outcome = perform(&tag(), ActionId::Export, &DeviceCapabilities::none()).unwrap();
        let ActionOutcome::Exported { contents, .. } = outcome else {
            panic!("expected an export");
        };

        let parsed = crate::parsers::ParsedFile::parse_nfc_struct(&contents).unwrap();
        assert_eq!(parsed.uid, "04:1E:23");
        assert_eq!(parsed.device_type, "ISO14443-3A");
    }

    #[test]
    fn an_exported_rfid_capture_round_trips_too() {
        let badge = Signal::new(
            SignalKind::LfRfid {
                protocol: "EM4100".to_string(),
                data: "12 34 56".to_string(),
            },
            0,
        );
        let ActionOutcome::Exported { contents, .. } =
            perform(&badge, ActionId::Export, &DeviceCapabilities::none()).unwrap()
        else {
            panic!("expected an export");
        };

        let parsed = crate::parsers::parse_rfid_struct(&contents).unwrap();
        assert_eq!(parsed.key_type, "EM4100");
        assert_eq!(parsed.data, "12 34 56");
    }

    #[test]
    fn transmitting_without_the_fap_says_what_is_missing() {
        // A silent no-op on Replay is the failure that teaches a user to
        // distrust the whole app.
        let outcome = perform(
            &subghz(Some(false)),
            ActionId::Replay,
            &DeviceCapabilities::none(),
        )
        .unwrap();
        match outcome {
            ActionOutcome::Unavailable { reason_key, .. } => {
                assert_eq!(reason_key, "action.needs_fap");
            }
            other => panic!("expected unavailable, got {:?}", other),
        }
    }

    #[test]
    fn transmitting_with_the_fap_admits_it_is_not_implemented_yet() {
        let outcome = perform(
            &subghz(Some(false)),
            ActionId::Replay,
            &DeviceCapabilities {
                fap_available: true,
                firmware: FirmwareCapabilities::from_version("Unleashed"),
            },
        )
        .unwrap();
        match outcome {
            ActionOutcome::Unavailable { reason_key, .. } => {
                assert_eq!(reason_key, "action.tx_not_implemented");
            }
            other => panic!("expected unavailable, got {:?}", other),
        }
    }

    #[test]
    fn a_rolling_code_remote_cannot_be_replayed_even_by_asking_directly() {
        // The UI hides the button; this is the backstop for a caller that does
        // not, so the rule lives in one place rather than only in the view.
        let err = perform(
            &subghz(Some(true)),
            ActionId::Replay,
            &DeviceCapabilities {
                fap_available: true,
                firmware: FirmwareCapabilities::from_version("Unleashed"),
            },
        )
        .unwrap_err();
        assert!(format!("{}", err).contains("not available"), "got {}", err);
    }

    #[test]
    fn a_bluetooth_sighting_cannot_be_emulated() {
        let device = Signal::new(
            SignalKind::Bluetooth {
                address: "AA:BB".to_string(),
                name: None,
            },
            0,
        );
        assert!(perform(&device, ActionId::Emulate, &DeviceCapabilities::none()).is_err());
    }

    #[test]
    fn comparison_asks_for_the_second_capture() {
        let outcome = perform(
            &subghz(None),
            ActionId::Compare,
            &DeviceCapabilities::none(),
        )
        .unwrap();
        assert!(matches!(outcome, ActionOutcome::Unavailable { .. }));
    }
}

#[cfg(test)]
mod firmware_gate_tests {
    use super::*;
    use crate::signals::firmware::FirmwareCapabilities;
    use crate::signals::model::Signal;

    fn remote_at(frequency_hz: u32) -> Signal {
        Signal::new(
            SignalKind::SubGhz {
                frequency_hz,
                preset: None,
                protocol: Some("Princeton".to_string()),
                rolling_code: Some(false),
            },
            0,
        )
    }

    fn device(fap: bool, version: &str) -> DeviceCapabilities {
        DeviceCapabilities {
            fap_available: fap,
            firmware: FirmwareCapabilities::from_version(version),
        }
    }

    #[test]
    fn stock_firmware_refuses_an_out_of_band_replay_before_trying() {
        // The device would decline silently, surfacing as a replay that seems to
        // work and changes nothing -- the most confusing failure available.
        let outcome = perform(
            &remote_at(500_000_000),
            ActionId::Replay,
            &device(true, "1.0.1 official"),
        )
        .unwrap();

        match outcome {
            ActionOutcome::Unavailable { reason_key, detail } => {
                assert_eq!(reason_key, "action.frequency_blocked");
                assert!(detail.contains("500.00 MHz"), "got {}", detail);
            }
            other => panic!("expected a refusal, got {:?}", other),
        }
    }

    #[test]
    fn stock_firmware_allows_a_replay_inside_its_bands() {
        let outcome = perform(
            &remote_at(433_920_000),
            ActionId::Replay,
            &device(true, "1.0.1 official"),
        )
        .unwrap();
        // Reaches the FAP check rather than the frequency one.
        match outcome {
            ActionOutcome::Unavailable { reason_key, .. } => {
                assert_eq!(reason_key, "action.tx_not_implemented");
            }
            other => panic!("expected the FAP limitation, got {:?}", other),
        }
    }

    #[test]
    fn unlocked_firmware_gets_past_the_frequency_gate() {
        let outcome = perform(
            &remote_at(500_000_000),
            ActionId::Replay,
            &device(true, "Unleashed 1.1"),
        )
        .unwrap();
        match outcome {
            ActionOutcome::Unavailable { reason_key, .. } => {
                assert_eq!(reason_key, "action.tx_not_implemented");
            }
            other => panic!("expected the FAP limitation, got {:?}", other),
        }
    }

    #[test]
    fn the_frequency_gate_does_not_touch_non_radio_actions() {
        // Saving or analysing an out-of-band capture is always fine; only
        // transmitting is restricted.
        for action in [ActionId::Save, ActionId::Analyze, ActionId::Export] {
            assert!(
                perform(&remote_at(500_000_000), action, &device(false, "official")).is_ok(),
                "{:?} must not be blocked by a transmit rule",
                action
            );
        }
    }

    #[test]
    fn an_emulated_tag_is_not_frequency_gated() {
        // NFC and iButton carry no Sub-GHz frequency; the gate must not invent
        // one and refuse them.
        let tag = Signal::new(
            SignalKind::Nfc {
                technology: "ISO14443-3A".to_string(),
                uid: "04:1E".to_string(),
                atqa: None,
                sak: None,
            },
            0,
        );
        match perform(&tag, ActionId::Emulate, &device(true, "official")).unwrap() {
            ActionOutcome::Unavailable { reason_key, .. } => {
                assert_eq!(reason_key, "action.tx_not_implemented");
            }
            other => panic!("expected the FAP limitation, got {:?}", other),
        }
    }
}
