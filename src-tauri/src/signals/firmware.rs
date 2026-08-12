//! What the connected Flipper can actually do.
//!
//! The app targets official firmware and adapts when it finds something else.
//! That adaptation has to be driven by what the device *reports*, never by what
//! the user says they installed: a wrong guess here either hides features that
//! work or offers ones that will fail, and both erode trust in everything else
//! the app claims.
//!
//! Custom firmwares differ from stock in ways that matter to the Radar --
//! chiefly which Sub-GHz frequencies they will transmit on. Detection is by
//! version string, because that is what `System.DeviceInfo` gives us.

use serde::{Deserialize, Serialize};

/// Which firmware family is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirmwareFamily {
    /// Official firmware from Flipper Devices.
    Official,
    Momentum,
    Unleashed,
    RogueMaster,
    /// Something we do not recognise. Treated as stock, which is the
    /// conservative choice: it never claims a capability we cannot confirm.
    Unknown,
}

impl FirmwareFamily {
    pub fn explain_key(self) -> &'static str {
        match self {
            FirmwareFamily::Official => "firmware.official",
            FirmwareFamily::Momentum => "firmware.momentum",
            FirmwareFamily::Unleashed => "firmware.unleashed",
            FirmwareFamily::RogueMaster => "firmware.roguemaster",
            FirmwareFamily::Unknown => "firmware.unknown",
        }
    }
}

/// Identify the firmware from the version string `System.DeviceInfo` reports.
///
/// Matching is case-insensitive and on substrings, because the custom builds
/// decorate their version strings freely and only the family name is reliable.
pub fn detect_family(version: &str) -> FirmwareFamily {
    let lowered = version.to_lowercase();

    // Checked before the others because RogueMaster's version strings often
    // also mention Unleashed, which it is derived from; the more specific name
    // has to win or every RogueMaster device is misidentified.
    if lowered.contains("roguemaster") || lowered.contains("rogue-master") {
        return FirmwareFamily::RogueMaster;
    }
    if lowered.contains("momentum") {
        return FirmwareFamily::Momentum;
    }
    if lowered.contains("unleashed") {
        return FirmwareFamily::Unleashed;
    }
    if lowered.contains("official") || lowered.contains("release") || lowered.contains("dev") {
        return FirmwareFamily::Official;
    }
    FirmwareFamily::Unknown
}

/// What this device permits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareCapabilities {
    pub family: FirmwareFamily,
    pub version: String,
    /// Whether transmission is restricted to the region's legal bands.
    ///
    /// Stock firmware refuses to transmit outside them. Custom builds generally
    /// do not, which is a legal question for the user, not a feature -- the app
    /// reports the fact and does not celebrate it.
    pub region_locked_tx: bool,
    /// Whether the firmware exposes extra Sub-GHz protocol decoders.
    pub extended_subghz_protocols: bool,
}

impl FirmwareCapabilities {
    /// Derive capabilities from a reported version string.
    pub fn from_version(version: &str) -> Self {
        let family = detect_family(version);
        // Unknown is deliberately treated as stock. Assuming the permissive
        // case would mean offering transmissions the device will silently
        // refuse, which looks like a broken app rather than a blocked band.
        let unlocked = matches!(
            family,
            FirmwareFamily::Momentum | FirmwareFamily::Unleashed | FirmwareFamily::RogueMaster
        );

        Self {
            family,
            version: version.to_string(),
            region_locked_tx: !unlocked,
            extended_subghz_protocols: unlocked,
        }
    }

    /// What to assume before a device has been identified.
    pub fn unknown() -> Self {
        Self {
            family: FirmwareFamily::Unknown,
            version: String::new(),
            region_locked_tx: true,
            extended_subghz_protocols: false,
        }
    }

    /// Whether this firmware will transmit on `frequency_hz`.
    ///
    /// Only the coarse question -- is it inside a band stock firmware allows --
    /// because the precise rules are regional and belong to the device.
    pub fn can_transmit_on(&self, frequency_hz: u32) -> bool {
        if !self.region_locked_tx {
            return true;
        }
        // The bands stock firmware will transmit in, as ranges in Hz.
        const ALLOWED: [(u32, u32); 3] = [
            (300_000_000, 348_000_000),
            (387_000_000, 464_000_000),
            (779_000_000, 928_000_000),
        ];
        ALLOWED
            .iter()
            .any(|(low, high)| frequency_hz >= *low && frequency_hz <= *high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_official_firmware() {
        assert_eq!(detect_family("1.0.1 official"), FirmwareFamily::Official);
        assert_eq!(detect_family("0.98.3-release"), FirmwareFamily::Official);
    }

    #[test]
    fn recognises_the_custom_families() {
        assert_eq!(detect_family("Momentum 009"), FirmwareFamily::Momentum);
        assert_eq!(detect_family("unleashed-1.1.2"), FirmwareFamily::Unleashed);
        assert_eq!(
            detect_family("RogueMaster 1234"),
            FirmwareFamily::RogueMaster
        );
    }

    #[test]
    fn roguemaster_wins_over_the_unleashed_it_is_built_on() {
        // RogueMaster version strings routinely mention Unleashed too. Checking
        // in the wrong order misidentifies every RogueMaster device.
        assert_eq!(
            detect_family("RogueMaster based on Unleashed"),
            FirmwareFamily::RogueMaster
        );
    }

    #[test]
    fn matching_ignores_case_and_decoration() {
        assert_eq!(detect_family("MOMENTUM-dev-abc"), FirmwareFamily::Momentum);
        assert_eq!(detect_family("  Unleashed  "), FirmwareFamily::Unleashed);
    }

    #[test]
    fn an_unrecognised_string_is_unknown_rather_than_a_guess() {
        assert_eq!(
            detect_family("something else entirely"),
            FirmwareFamily::Unknown
        );
        assert_eq!(detect_family(""), FirmwareFamily::Unknown);
    }

    #[test]
    fn unknown_firmware_is_treated_as_locked_down() {
        // Assuming the permissive case would offer transmissions the device
        // refuses, which reads as a broken app rather than a blocked band.
        let capabilities = FirmwareCapabilities::from_version("mystery build");
        assert!(capabilities.region_locked_tx);
        assert!(!capabilities.extended_subghz_protocols);
    }

    #[test]
    fn stock_firmware_is_region_locked() {
        let capabilities = FirmwareCapabilities::from_version("1.0.1 official");
        assert!(capabilities.region_locked_tx);
        assert!(!capabilities.extended_subghz_protocols);
    }

    #[test]
    fn custom_firmware_reports_the_extra_freedom_it_has() {
        for version in ["Momentum 009", "Unleashed 1.1", "RogueMaster x"] {
            let capabilities = FirmwareCapabilities::from_version(version);
            assert!(!capabilities.region_locked_tx, "{}", version);
            assert!(capabilities.extended_subghz_protocols, "{}", version);
        }
    }

    #[test]
    fn stock_firmware_refuses_frequencies_outside_its_bands() {
        let stock = FirmwareCapabilities::from_version("1.0.1 official");
        assert!(
            stock.can_transmit_on(433_920_000),
            "433.92 is inside a band"
        );
        assert!(
            stock.can_transmit_on(868_350_000),
            "868.35 is inside a band"
        );
        assert!(
            !stock.can_transmit_on(500_000_000),
            "500 MHz is between bands"
        );
        assert!(
            !stock.can_transmit_on(100_000_000),
            "FM radio is far outside"
        );
    }

    #[test]
    fn unlocked_firmware_transmits_anywhere_it_is_asked() {
        let unlocked = FirmwareCapabilities::from_version("Unleashed");
        assert!(unlocked.can_transmit_on(500_000_000));
        assert!(unlocked.can_transmit_on(100_000_000));
    }

    #[test]
    fn band_edges_are_inclusive() {
        let stock = FirmwareCapabilities::from_version("official");
        assert!(stock.can_transmit_on(300_000_000));
        assert!(stock.can_transmit_on(348_000_000));
        assert!(!stock.can_transmit_on(299_999_999));
        assert!(!stock.can_transmit_on(348_000_001));
    }

    #[test]
    fn the_pre_connection_default_assumes_nothing() {
        let unknown = FirmwareCapabilities::unknown();
        assert_eq!(unknown.family, FirmwareFamily::Unknown);
        assert!(unknown.region_locked_tx);
        assert!(unknown.version.is_empty());
    }

    #[test]
    fn every_family_has_its_own_explanation_key() {
        let families = [
            FirmwareFamily::Official,
            FirmwareFamily::Momentum,
            FirmwareFamily::Unleashed,
            FirmwareFamily::RogueMaster,
            FirmwareFamily::Unknown,
        ];
        let mut keys: Vec<&str> = families.iter().map(|f| f.explain_key()).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count, "duplicate firmware explain key");
    }
}
