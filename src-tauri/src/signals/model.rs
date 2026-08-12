//! The unified signal model behind the Signal Radar.
//!
//! Every radio on the Flipper reports something structurally different -- a
//! Sub-GHz burst, an NFC tag UID, a 1-Wire serial number, an IR frame -- but the
//! Radar shows them side by side and answers the same three questions for each:
//! what is it, how many have been seen, and what can be done with it. [`Signal`]
//! is the shape that makes the UI chip-agnostic, so adding a radio does not mean
//! adding a parallel screen.
//!
//! Human-readable explanations deliberately live outside this module. Rust
//! carries only stable `explain_key`s; the prose sits in the frontend i18n
//! files, so wording and translations change without recompiling.

use serde::{Deserialize, Serialize};

/// A radio or peripheral on (or attached to) the Flipper.
///
/// Flat rather than nested so it stores as a single column and crosses the IPC
/// boundary as a plain string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chip {
    /// CC1101, 300-928 MHz.
    SubGhz,
    /// ST25R3916, 13.56 MHz.
    Nfc,
    /// 125 kHz low-frequency RFID.
    LfRfid,
    /// 1-Wire iButton.
    IButton,
    Infrared,
    Bluetooth,
    Gpio,
    /// ESP32 WiFi developer board on the GPIO header.
    WifiDevBoard,
    /// A second CC1101 attached externally.
    ExternalCc1101,
    /// NRF24 module on the GPIO header.
    Nrf24,
    /// Something answered on the header that we could not identify.
    UnknownModule,
}

impl Chip {
    /// Stable key for the "what is this chip?" copy shown in the UI.
    pub fn explain_key(self) -> &'static str {
        match self {
            Chip::SubGhz => "chip.subghz",
            Chip::Nfc => "chip.nfc",
            Chip::LfRfid => "chip.lfrfid",
            Chip::IButton => "chip.ibutton",
            Chip::Infrared => "chip.infrared",
            Chip::Bluetooth => "chip.bluetooth",
            Chip::Gpio => "chip.gpio",
            Chip::WifiDevBoard => "chip.wifi_devboard",
            Chip::ExternalCc1101 => "chip.external_cc1101",
            Chip::Nrf24 => "chip.nrf24",
            Chip::UnknownModule => "chip.unknown_module",
        }
    }

    /// Whether this chip is physically attached rather than built in.
    ///
    /// The Radar only draws these once they have actually been detected, so an
    /// empty GPIO header does not show phantom hardware.
    pub fn is_external_module(self) -> bool {
        matches!(
            self,
            Chip::WifiDevBoard | Chip::ExternalCc1101 | Chip::Nrf24 | Chip::UnknownModule
        )
    }

    /// Short stable identifier used in storage and as an SVG hotspot id.
    pub fn slug(self) -> &'static str {
        match self {
            Chip::SubGhz => "subghz",
            Chip::Nfc => "nfc",
            Chip::LfRfid => "lfrfid",
            Chip::IButton => "ibutton",
            Chip::Infrared => "infrared",
            Chip::Bluetooth => "bluetooth",
            Chip::Gpio => "gpio",
            Chip::WifiDevBoard => "wifi_devboard",
            Chip::ExternalCc1101 => "external_cc1101",
            Chip::Nrf24 => "nrf24",
            Chip::UnknownModule => "unknown_module",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Chip> {
        Some(match slug {
            "subghz" => Chip::SubGhz,
            "nfc" => Chip::Nfc,
            "lfrfid" => Chip::LfRfid,
            "ibutton" => Chip::IButton,
            "infrared" => Chip::Infrared,
            "bluetooth" => Chip::Bluetooth,
            "gpio" => Chip::Gpio,
            "wifi_devboard" => Chip::WifiDevBoard,
            "external_cc1101" => Chip::ExternalCc1101,
            "nrf24" => Chip::Nrf24,
            "unknown_module" => Chip::UnknownModule,
            _ => return None,
        })
    }

    /// Every chip the Radar can draw, in the order it reads best.
    pub fn all() -> &'static [Chip] {
        &[
            Chip::SubGhz,
            Chip::Nfc,
            Chip::LfRfid,
            Chip::IButton,
            Chip::Infrared,
            Chip::Bluetooth,
            Chip::Gpio,
            Chip::WifiDevBoard,
            Chip::ExternalCc1101,
            Chip::Nrf24,
            Chip::UnknownModule,
        ]
    }
}

/// Per-chip detail. The variant determines which fields make sense.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalKind {
    SubGhz {
        frequency_hz: u32,
        preset: Option<String>,
        protocol: Option<String>,
        /// `Some(true)` for rolling-code remotes, which cannot be usefully
        /// replayed. The UI must say so instead of offering a replay that
        /// silently does nothing.
        rolling_code: Option<bool>,
    },
    Nfc {
        technology: String,
        uid: String,
        atqa: Option<String>,
        sak: Option<String>,
    },
    LfRfid {
        protocol: String,
        data: String,
    },
    IButton {
        protocol: String,
        id: String,
    },
    Infrared {
        protocol: Option<String>,
        address: Option<String>,
        command: Option<String>,
    },
    Bluetooth {
        address: String,
        name: Option<String>,
    },
    Gpio {
        pin: String,
        level: bool,
    },
    /// An attached module announcing itself.
    Module {
        module: String,
        detail: Option<String>,
    },
}

impl SignalKind {
    /// Which chip produced this kind of signal.
    pub fn chip(&self) -> Chip {
        match self {
            SignalKind::SubGhz { .. } => Chip::SubGhz,
            SignalKind::Nfc { .. } => Chip::Nfc,
            SignalKind::LfRfid { .. } => Chip::LfRfid,
            SignalKind::IButton { .. } => Chip::IButton,
            SignalKind::Infrared { .. } => Chip::Infrared,
            SignalKind::Bluetooth { .. } => Chip::Bluetooth,
            SignalKind::Gpio { .. } => Chip::Gpio,
            SignalKind::Module { .. } => Chip::UnknownModule,
        }
    }

    /// Key for the plain-language explanation of this specific signal.
    ///
    /// More specific than the chip's own key where the distinction matters to a
    /// beginner -- a rolling-code remote behaves nothing like a fixed-code one.
    pub fn explain_key(&self) -> &'static str {
        match self {
            SignalKind::SubGhz {
                rolling_code: Some(true),
                ..
            } => "signal.subghz.rolling",
            SignalKind::SubGhz {
                rolling_code: Some(false),
                ..
            } => "signal.subghz.fixed",
            SignalKind::SubGhz { .. } => "signal.subghz.unknown",
            SignalKind::Nfc { .. } => "signal.nfc",
            SignalKind::LfRfid { .. } => "signal.lfrfid",
            SignalKind::IButton { .. } => "signal.ibutton",
            SignalKind::Infrared { .. } => "signal.infrared",
            SignalKind::Bluetooth { .. } => "signal.bluetooth",
            SignalKind::Gpio { .. } => "signal.gpio",
            SignalKind::Module { .. } => "signal.module",
        }
    }

    /// The fields that identify this signal across sightings.
    ///
    /// Deliberately excludes anything that varies between encounters (RSSI,
    /// timestamps, location); those describe *a sighting*, not the signal.
    fn identity(&self) -> String {
        match self {
            SignalKind::SubGhz {
                frequency_hz,
                protocol,
                ..
            } => format!(
                "subghz|{}|{}",
                frequency_hz,
                protocol.as_deref().unwrap_or("")
            ),
            SignalKind::Nfc {
                technology, uid, ..
            } => format!("nfc|{}|{}", technology, uid),
            SignalKind::LfRfid { protocol, data } => format!("lfrfid|{}|{}", protocol, data),
            SignalKind::IButton { protocol, id } => format!("ibutton|{}|{}", protocol, id),
            SignalKind::Infrared {
                protocol,
                address,
                command,
            } => format!(
                "infrared|{}|{}|{}",
                protocol.as_deref().unwrap_or(""),
                address.as_deref().unwrap_or(""),
                command.as_deref().unwrap_or("")
            ),
            SignalKind::Bluetooth { address, .. } => format!("bluetooth|{}", address),
            SignalKind::Gpio { pin, .. } => format!("gpio|{}", pin),
            SignalKind::Module { module, .. } => format!("module|{}", module),
        }
    }
}

/// Where a sighting happened. Optional throughout -- location is a user opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

/// Something the user can do with a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionId {
    /// Store the capture on the Flipper's SD card.
    Save,
    /// Re-transmit it (Sub-GHz, IR).
    Replay,
    /// Present it to a reader (NFC, iButton, LF RFID).
    Emulate,
    /// Run the existing reverse-engineering analysis over the raw bytes.
    Analyze,
    /// Compare against another capture.
    Compare,
    /// Export off the device.
    Export,
}

impl ActionId {
    /// Whether performing this action makes the Flipper transmit.
    ///
    /// Transmitting is what carries legal weight, so this is the flag that gates
    /// the ownership confirmation rather than the chip or signal type.
    pub fn requires_transmission(self) -> bool {
        matches!(self, ActionId::Replay | ActionId::Emulate)
    }

    pub fn explain_key(self) -> &'static str {
        match self {
            ActionId::Save => "action.save",
            ActionId::Replay => "action.replay",
            ActionId::Emulate => "action.emulate",
            ActionId::Analyze => "action.analyze",
            ActionId::Compare => "action.compare",
            ActionId::Export => "action.export",
        }
    }
}

/// One distinct signal, aggregated across every time it has been seen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    /// Stable fingerprint. Two encounters with the same remote produce the same
    /// id, which is what lets the Radar answer "how many are there" with a count
    /// rather than a pile of duplicates.
    pub id: String,
    pub chip: Chip,
    pub kind: SignalKind,
    /// Unix seconds.
    pub first_seen: i64,
    pub last_seen: i64,
    pub count: u32,
    pub rssi_dbm: Option<i16>,
    pub location: Option<GeoPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw: Vec<u8>,
}

impl Signal {
    /// Build a signal from its first sighting.
    pub fn new(kind: SignalKind, seen_at: i64) -> Self {
        Self {
            id: fingerprint(&kind),
            chip: kind.chip(),
            kind,
            first_seen: seen_at,
            last_seen: seen_at,
            count: 1,
            rssi_dbm: None,
            location: None,
            raw: Vec::new(),
        }
    }

    pub fn with_rssi(mut self, rssi_dbm: i16) -> Self {
        self.rssi_dbm = Some(rssi_dbm);
        self
    }

    pub fn with_location(mut self, location: GeoPoint) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_raw(mut self, raw: Vec<u8>) -> Self {
        self.raw = raw;
        self
    }

    /// Fold in another sighting of the same signal.
    ///
    /// `first_seen` is kept as the earliest and `last_seen` as the latest rather
    /// than assuming sightings arrive in order -- replayed history and live
    /// events can interleave.
    pub fn record_sighting(&mut self, seen_at: i64, rssi_dbm: Option<i16>) {
        self.count = self.count.saturating_add(1);
        self.first_seen = self.first_seen.min(seen_at);
        self.last_seen = self.last_seen.max(seen_at);
        if rssi_dbm.is_some() {
            self.rssi_dbm = rssi_dbm;
        }
    }

    /// What the user can do with this signal.
    pub fn available_actions(&self) -> Vec<ActionId> {
        let mut actions = vec![
            ActionId::Save,
            ActionId::Analyze,
            ActionId::Compare,
            ActionId::Export,
        ];

        match &self.kind {
            // A rolling-code remote changes its code every press, so replaying a
            // captured one does nothing. Offering the button would teach the
            // user the wrong model of how their gate works.
            SignalKind::SubGhz {
                rolling_code: Some(true),
                ..
            } => {}
            SignalKind::SubGhz { .. } | SignalKind::Infrared { .. } => {
                actions.push(ActionId::Replay)
            }
            SignalKind::Nfc { .. } | SignalKind::IButton { .. } | SignalKind::LfRfid { .. } => {
                actions.push(ActionId::Emulate)
            }
            // Observation only: nothing to transmit back.
            SignalKind::Bluetooth { .. } | SignalKind::Gpio { .. } | SignalKind::Module { .. } => {}
        }

        actions
    }

    pub fn explain_key(&self) -> &'static str {
        self.kind.explain_key()
    }
}

/// Stable identifier for a signal's identifying fields.
///
/// FNV-1a: short, dependency-free, and stable across runs and platforms, which
/// is what matters here. This is a de-duplication key, never a security
/// primitive.
pub fn fingerprint(kind: &SignalKind) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in kind.identity().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{}-{:016x}", kind.chip().slug(), hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subghz(frequency_hz: u32, rolling: Option<bool>) -> SignalKind {
        SignalKind::SubGhz {
            frequency_hz,
            preset: Some("AM650".to_string()),
            protocol: Some("Princeton".to_string()),
            rolling_code: rolling,
        }
    }

    #[test]
    fn same_signal_seen_twice_has_the_same_id() {
        let a = Signal::new(subghz(433_920_000, Some(false)), 100);
        let b = Signal::new(subghz(433_920_000, Some(false)), 900);
        assert_eq!(
            a.id, b.id,
            "identical remotes must aggregate, not duplicate"
        );
    }

    #[test]
    fn a_different_frequency_is_a_different_signal() {
        let a = Signal::new(subghz(433_920_000, Some(false)), 100);
        let b = Signal::new(subghz(868_350_000, Some(false)), 100);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn transient_measurements_do_not_change_identity() {
        // RSSI, time and place describe a sighting, not the signal itself.
        let base = Signal::new(subghz(433_920_000, None), 10);
        let noisy = Signal::new(subghz(433_920_000, None), 9999)
            .with_rssi(-91)
            .with_location(GeoPoint {
                latitude: 45.07,
                longitude: 7.68,
            });
        assert_eq!(base.id, noisy.id);
    }

    #[test]
    fn ids_are_namespaced_by_chip() {
        let signal = Signal::new(subghz(433_920_000, None), 0);
        assert!(signal.id.starts_with("subghz-"), "got {}", signal.id);
    }

    #[test]
    fn recording_a_sighting_counts_and_widens_the_window() {
        let mut signal = Signal::new(subghz(433_920_000, None), 500);
        signal.record_sighting(900, Some(-60));
        // Out-of-order arrival must move first_seen backwards, not corrupt it.
        signal.record_sighting(100, None);

        assert_eq!(signal.count, 3);
        assert_eq!(signal.first_seen, 100);
        assert_eq!(signal.last_seen, 900);
        assert_eq!(signal.rssi_dbm, Some(-60));
    }

    #[test]
    fn rolling_code_signals_are_not_offered_a_replay() {
        let rolling = Signal::new(subghz(433_920_000, Some(true)), 0);
        assert!(
            !rolling.available_actions().contains(&ActionId::Replay),
            "replaying a rolling code does nothing and would mislead the user"
        );

        let fixed = Signal::new(subghz(433_920_000, Some(false)), 0);
        assert!(fixed.available_actions().contains(&ActionId::Replay));
    }

    #[test]
    fn rolling_and_fixed_remotes_explain_themselves_differently() {
        let rolling = Signal::new(subghz(433_920_000, Some(true)), 0);
        let fixed = Signal::new(subghz(433_920_000, Some(false)), 0);
        assert_ne!(rolling.explain_key(), fixed.explain_key());
    }

    #[test]
    fn tags_are_emulated_and_never_replayed() {
        let nfc = Signal::new(
            SignalKind::Nfc {
                technology: "ISO14443-3A".to_string(),
                uid: "04A2B3C4D5".to_string(),
                atqa: None,
                sak: None,
            },
            0,
        );
        let actions = nfc.available_actions();
        assert!(actions.contains(&ActionId::Emulate));
        assert!(!actions.contains(&ActionId::Replay));
    }

    #[test]
    fn observation_only_signals_offer_nothing_that_transmits() {
        for kind in [
            SignalKind::Bluetooth {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                name: Some("Headphones".to_string()),
            },
            SignalKind::Gpio {
                pin: "PA7".to_string(),
                level: true,
            },
            SignalKind::Module {
                module: "esp32-s2".to_string(),
                detail: None,
            },
        ] {
            let signal = Signal::new(kind, 0);
            assert!(
                !signal
                    .available_actions()
                    .iter()
                    .any(|a| a.requires_transmission()),
                "{:?} must not offer a transmitting action",
                signal.chip
            );
        }
    }

    #[test]
    fn only_replay_and_emulate_are_gated_as_transmissions() {
        assert!(ActionId::Replay.requires_transmission());
        assert!(ActionId::Emulate.requires_transmission());
        for action in [
            ActionId::Save,
            ActionId::Analyze,
            ActionId::Compare,
            ActionId::Export,
        ] {
            assert!(!action.requires_transmission(), "{:?}", action);
        }
    }

    #[test]
    fn every_chip_round_trips_through_its_slug() {
        for chip in Chip::all() {
            assert_eq!(Chip::from_slug(chip.slug()), Some(*chip));
        }
    }

    #[test]
    fn chip_slugs_and_explain_keys_are_unique() {
        let mut slugs: Vec<&str> = Chip::all().iter().map(|c| c.slug()).collect();
        slugs.sort_unstable();
        let count = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), count, "duplicate chip slug");

        let mut keys: Vec<&str> = Chip::all().iter().map(|c| c.explain_key()).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count, "duplicate explain key");
    }

    #[test]
    fn external_modules_are_flagged_so_the_radar_can_hide_them_until_detected() {
        assert!(Chip::WifiDevBoard.is_external_module());
        assert!(Chip::Nrf24.is_external_module());
        assert!(!Chip::SubGhz.is_external_module());
        assert!(!Chip::Nfc.is_external_module());
    }

    #[test]
    fn kind_reports_its_own_chip() {
        assert_eq!(subghz(433_920_000, None).chip(), Chip::SubGhz);
        assert_eq!(
            SignalKind::IButton {
                protocol: "DS1990".to_string(),
                id: "01:02:03".to_string(),
            }
            .chip(),
            Chip::IButton
        );
    }
}
