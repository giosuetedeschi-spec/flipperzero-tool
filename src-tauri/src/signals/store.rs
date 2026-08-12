//! Persistence for detected signals.
//!
//! Two tables, because a signal and an encounter with it are different things.
//! `signals` holds one row per distinct signal, keyed by its fingerprint;
//! `signal_sightings` holds one row per encounter. That split is what makes
//! "seen 14 times, first last Tuesday, strongest near the garage" answerable --
//! collapsing sightings into a counter would throw away the history the Radar
//! is meant to show.
//!
//! Follows the shape of [`crate::vfs`]: the logic takes a plain `&Connection`
//! so it can be tested against in-memory SQLite with no app handle.

use super::model::{Chip, GeoPoint, Signal, SignalKind};
use crate::errors::AppError;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Create the signal tables if they are absent.
///
/// Safe to call on an existing database; runs alongside the `vfs` schema in the
/// same file rather than opening a second one.
pub fn init_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS signals (
            id          TEXT PRIMARY KEY,
            chip        TEXT NOT NULL,
            kind_json   TEXT NOT NULL,
            first_seen  INTEGER NOT NULL,
            last_seen   INTEGER NOT NULL,
            count       INTEGER NOT NULL DEFAULT 1,
            rssi_dbm    INTEGER,
            latitude    REAL,
            longitude   REAL,
            raw         BLOB
        );
        CREATE INDEX IF NOT EXISTS idx_signals_chip ON signals(chip);
        CREATE INDEX IF NOT EXISTS idx_signals_last_seen ON signals(last_seen);

        CREATE TABLE IF NOT EXISTS signal_sightings (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            signal_id   TEXT NOT NULL,
            seen_at     INTEGER NOT NULL,
            rssi_dbm    INTEGER,
            latitude    REAL,
            longitude   REAL,
            FOREIGN KEY (signal_id) REFERENCES signals(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sightings_signal ON signal_sightings(signal_id);",
    )
    .map_err(AppError::from)
}

/// Record one detection.
///
/// Inserts the signal on first sight and folds the encounter into the existing
/// row afterwards, so the caller does not have to know which case it is in.
/// Returns `true` when the signal had never been seen before -- the Radar uses
/// that to decide whether to pulse the chip and notify.
pub fn record_conn(conn: &Connection, signal: &Signal) -> Result<bool, AppError> {
    let kind_json = serde_json::to_string(&signal.kind)
        .map_err(|e| AppError::ParseError(format!("Cannot serialize signal kind: {}", e)))?;

    let existing: Option<(i64, i64, u32)> = conn
        .query_row(
            "SELECT first_seen, last_seen, count FROM signals WHERE id = ?1",
            params![signal.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(AppError::from)?;

    let is_new = existing.is_none();

    match existing {
        None => {
            conn.execute(
                "INSERT INTO signals
                   (id, chip, kind_json, first_seen, last_seen, count, rssi_dbm, latitude, longitude, raw)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9)",
                params![
                    signal.id,
                    signal.chip.slug(),
                    kind_json,
                    signal.first_seen,
                    signal.last_seen,
                    signal.rssi_dbm,
                    signal.location.map(|l| l.latitude),
                    signal.location.map(|l| l.longitude),
                    signal.raw,
                ],
            )
            .map_err(AppError::from)?;
        }
        Some((first_seen, last_seen, count)) => {
            // Sightings can arrive out of order (live events interleaved with
            // replayed history), so widen the window rather than overwrite it.
            conn.execute(
                "UPDATE signals
                    SET first_seen = ?2, last_seen = ?3, count = ?4,
                        rssi_dbm = COALESCE(?5, rssi_dbm),
                        latitude = COALESCE(?6, latitude),
                        longitude = COALESCE(?7, longitude)
                  WHERE id = ?1",
                params![
                    signal.id,
                    first_seen.min(signal.first_seen),
                    last_seen.max(signal.last_seen),
                    count.saturating_add(1),
                    signal.rssi_dbm,
                    signal.location.map(|l| l.latitude),
                    signal.location.map(|l| l.longitude),
                ],
            )
            .map_err(AppError::from)?;
        }
    }

    conn.execute(
        "INSERT INTO signal_sightings (signal_id, seen_at, rssi_dbm, latitude, longitude)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            signal.id,
            signal.last_seen,
            signal.rssi_dbm,
            signal.location.map(|l| l.latitude),
            signal.location.map(|l| l.longitude),
        ],
    )
    .map_err(AppError::from)?;

    Ok(is_new)
}

fn row_to_signal(row: &rusqlite::Row<'_>) -> rusqlite::Result<Signal> {
    let chip_slug: String = row.get("chip")?;
    let kind_json: String = row.get("kind_json")?;
    let latitude: Option<f64> = row.get("latitude")?;
    let longitude: Option<f64> = row.get("longitude")?;

    // A row whose kind cannot be parsed is corrupt rather than merely empty;
    // surface it instead of inventing a placeholder signal.
    let kind: SignalKind = serde_json::from_str(&kind_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Signal {
        id: row.get("id")?,
        chip: Chip::from_slug(&chip_slug).unwrap_or(Chip::UnknownModule),
        kind,
        first_seen: row.get("first_seen")?,
        last_seen: row.get("last_seen")?,
        count: row.get("count")?,
        rssi_dbm: row.get("rssi_dbm")?,
        location: match (latitude, longitude) {
            (Some(latitude), Some(longitude)) => Some(GeoPoint {
                latitude,
                longitude,
            }),
            _ => None,
        },
        raw: row.get::<_, Option<Vec<u8>>>("raw")?.unwrap_or_default(),
    })
}

/// Every signal seen for one chip, most recently seen first.
pub fn list_by_chip_conn(conn: &Connection, chip: Chip) -> Result<Vec<Signal>, AppError> {
    let mut stmt = conn
        .prepare("SELECT * FROM signals WHERE chip = ?1 ORDER BY last_seen DESC")
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![chip.slug()], row_to_signal)
        .map_err(AppError::from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

/// How many distinct signals each chip has seen.
///
/// This is what the Radar prints on each chip badge, so it counts signals, not
/// sightings: "3 remotes nearby", not "seen 240 bursts".
pub fn counts_by_chip_conn(conn: &Connection) -> Result<Vec<(Chip, u32)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT chip, COUNT(*) FROM signals GROUP BY chip")
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map([], |row| {
            let slug: String = row.get(0)?;
            let count: u32 = row.get(1)?;
            Ok((slug, count))
        })
        .map_err(AppError::from)?;

    let mut out = Vec::new();
    for row in rows {
        let (slug, count) = row.map_err(AppError::from)?;
        if let Some(chip) = Chip::from_slug(&slug) {
            out.push((chip, count));
        }
    }
    Ok(out)
}

/// One recorded encounter with a signal.
///
/// Distinct from [`Signal`] on purpose: the signal is the thing, this is a time
/// and place it was observed. The Radar's timeline and RSSI history are built
/// from these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sighting {
    /// Unix seconds.
    pub seen_at: i64,
    pub rssi_dbm: Option<i16>,
    pub location: Option<GeoPoint>,
}

/// Every recorded encounter with one signal, oldest first.
pub fn sightings_conn(conn: &Connection, signal_id: &str) -> Result<Vec<Sighting>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT seen_at, rssi_dbm, latitude, longitude
               FROM signal_sightings WHERE signal_id = ?1 ORDER BY seen_at ASC",
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![signal_id], |row| {
            let latitude: Option<f64> = row.get(2)?;
            let longitude: Option<f64> = row.get(3)?;
            Ok(Sighting {
                seen_at: row.get(0)?,
                rssi_dbm: row.get(1)?,
                location: match (latitude, longitude) {
                    (Some(latitude), Some(longitude)) => Some(GeoPoint {
                        latitude,
                        longitude,
                    }),
                    _ => None,
                },
            })
        })
        .map_err(AppError::from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn get_conn(conn: &Connection, signal_id: &str) -> Result<Option<Signal>, AppError> {
    conn.query_row(
        "SELECT * FROM signals WHERE id = ?1",
        params![signal_id],
        row_to_signal,
    )
    .optional()
    .map_err(AppError::from)
}

/// Forget one signal and its history.
pub fn delete_conn(conn: &Connection, signal_id: &str) -> Result<bool, AppError> {
    // The foreign key cascade only fires when enforcement is on, which is off by
    // default in SQLite, so remove the sightings explicitly.
    conn.execute(
        "DELETE FROM signal_sightings WHERE signal_id = ?1",
        params![signal_id],
    )
    .map_err(AppError::from)?;
    let removed = conn
        .execute("DELETE FROM signals WHERE id = ?1", params![signal_id])
        .map_err(AppError::from)?;
    Ok(removed > 0)
}

pub fn clear_conn(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("DELETE FROM signal_sightings; DELETE FROM signals;")
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn remote(frequency_hz: u32) -> SignalKind {
        SignalKind::SubGhz {
            frequency_hz,
            preset: Some("AM650".to_string()),
            protocol: Some("Princeton".to_string()),
            rolling_code: Some(false),
        }
    }

    fn tag(uid: &str) -> SignalKind {
        SignalKind::Nfc {
            technology: "ISO14443-3A".to_string(),
            uid: uid.to_string(),
            atqa: None,
            sak: None,
        }
    }

    #[test]
    fn init_schema_is_idempotent() {
        let conn = test_conn();
        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap();
    }

    #[test]
    fn first_sighting_is_reported_as_new() {
        let conn = test_conn();
        let signal = Signal::new(remote(433_920_000), 1_000);
        assert!(record_conn(&conn, &signal).unwrap());
    }

    #[test]
    fn seeing_the_same_signal_again_increments_instead_of_duplicating() {
        let conn = test_conn();
        let first = Signal::new(remote(433_920_000), 1_000).with_rssi(-70);
        let again = Signal::new(remote(433_920_000), 2_000).with_rssi(-55);

        assert!(record_conn(&conn, &first).unwrap());
        assert!(
            !record_conn(&conn, &again).unwrap(),
            "second sighting is not new"
        );

        let stored = list_by_chip_conn(&conn, Chip::SubGhz).unwrap();
        assert_eq!(stored.len(), 1, "must aggregate, not duplicate");
        assert_eq!(stored[0].count, 2);
        assert_eq!(stored[0].first_seen, 1_000);
        assert_eq!(stored[0].last_seen, 2_000);
        assert_eq!(stored[0].rssi_dbm, Some(-55), "latest reading wins");
    }

    #[test]
    fn out_of_order_sightings_widen_the_window_rather_than_corrupt_it() {
        let conn = test_conn();
        record_conn(&conn, &Signal::new(remote(433_920_000), 5_000)).unwrap();
        record_conn(&conn, &Signal::new(remote(433_920_000), 1_000)).unwrap();

        let stored = &list_by_chip_conn(&conn, Chip::SubGhz).unwrap()[0];
        assert_eq!(stored.first_seen, 1_000);
        assert_eq!(stored.last_seen, 5_000);
    }

    #[test]
    fn every_sighting_is_kept_as_history() {
        let conn = test_conn();
        let signal = Signal::new(remote(433_920_000), 1_000);
        for t in [1_000, 1_100, 1_200] {
            let mut s = signal.clone();
            s.first_seen = t;
            s.last_seen = t;
            record_conn(&conn, &s).unwrap();
        }

        let history = sightings_conn(&conn, &signal.id).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].seen_at, 1_000);
        assert_eq!(history[2].seen_at, 1_200);
    }

    #[test]
    fn location_is_stored_and_read_back() {
        let conn = test_conn();
        let signal = Signal::new(remote(433_920_000), 1_000).with_location(GeoPoint {
            latitude: 45.0703,
            longitude: 7.6869,
        });
        record_conn(&conn, &signal).unwrap();

        let stored = get_conn(&conn, &signal.id).unwrap().unwrap();
        let location = stored.location.expect("location should persist");
        assert!((location.latitude - 45.0703).abs() < 1e-9);
        assert!((location.longitude - 7.6869).abs() < 1e-9);
    }

    #[test]
    fn a_signal_without_a_location_stays_without_one() {
        let conn = test_conn();
        let signal = Signal::new(remote(433_920_000), 1_000);
        record_conn(&conn, &signal).unwrap();
        assert!(
            get_conn(&conn, &signal.id)
                .unwrap()
                .unwrap()
                .location
                .is_none()
        );
    }

    #[test]
    fn a_later_sighting_does_not_erase_a_known_location() {
        // COALESCE keeps the last known fix when a sighting has no GPS.
        let conn = test_conn();
        let located = Signal::new(remote(433_920_000), 1_000).with_location(GeoPoint {
            latitude: 45.0,
            longitude: 7.0,
        });
        record_conn(&conn, &located).unwrap();
        record_conn(&conn, &Signal::new(remote(433_920_000), 2_000)).unwrap();

        assert!(
            get_conn(&conn, &located.id)
                .unwrap()
                .unwrap()
                .location
                .is_some()
        );
    }

    #[test]
    fn kinds_survive_the_round_trip() {
        let conn = test_conn();
        let signal = Signal::new(tag("04A2B3C4D5"), 1_000);
        record_conn(&conn, &signal).unwrap();

        let stored = get_conn(&conn, &signal.id).unwrap().unwrap();
        assert_eq!(stored.kind, signal.kind);
        assert_eq!(stored.chip, Chip::Nfc);
    }

    #[test]
    fn raw_bytes_survive_the_round_trip() {
        let conn = test_conn();
        let raw: Vec<u8> = vec![0x00, 0xFF, 0x80, 0x7F];
        let signal = Signal::new(remote(433_920_000), 1_000).with_raw(raw.clone());
        record_conn(&conn, &signal).unwrap();

        assert_eq!(get_conn(&conn, &signal.id).unwrap().unwrap().raw, raw);
    }

    #[test]
    fn counts_are_per_chip_and_count_signals_not_sightings() {
        let conn = test_conn();
        // One remote seen three times, plus two distinct tags.
        for t in [1, 2, 3] {
            record_conn(&conn, &Signal::new(remote(433_920_000), t)).unwrap();
        }
        record_conn(&conn, &Signal::new(tag("AAAA"), 1)).unwrap();
        record_conn(&conn, &Signal::new(tag("BBBB"), 1)).unwrap();

        let counts = counts_by_chip_conn(&conn).unwrap();
        assert_eq!(
            counts.iter().find(|(c, _)| *c == Chip::SubGhz).unwrap().1,
            1,
            "three sightings of one remote is still one signal"
        );
        assert_eq!(counts.iter().find(|(c, _)| *c == Chip::Nfc).unwrap().1, 2);
    }

    #[test]
    fn listing_is_newest_first() {
        let conn = test_conn();
        record_conn(&conn, &Signal::new(remote(433_920_000), 1_000)).unwrap();
        record_conn(&conn, &Signal::new(remote(868_350_000), 9_000)).unwrap();

        let stored = list_by_chip_conn(&conn, Chip::SubGhz).unwrap();
        assert_eq!(stored[0].last_seen, 9_000);
    }

    #[test]
    fn chips_with_nothing_seen_are_simply_absent() {
        let conn = test_conn();
        record_conn(&conn, &Signal::new(remote(433_920_000), 1)).unwrap();
        let counts = counts_by_chip_conn(&conn).unwrap();
        assert!(counts.iter().all(|(c, _)| *c != Chip::IButton));
    }

    #[test]
    fn deleting_a_signal_takes_its_history_with_it() {
        let conn = test_conn();
        let signal = Signal::new(remote(433_920_000), 1_000);
        record_conn(&conn, &signal).unwrap();
        record_conn(&conn, &signal).unwrap();

        assert!(delete_conn(&conn, &signal.id).unwrap());
        assert!(get_conn(&conn, &signal.id).unwrap().is_none());
        assert!(sightings_conn(&conn, &signal.id).unwrap().is_empty());
    }

    #[test]
    fn deleting_something_absent_reports_false_rather_than_failing() {
        let conn = test_conn();
        assert!(!delete_conn(&conn, "subghz-doesnotexist").unwrap());
    }

    #[test]
    fn clearing_empties_both_tables() {
        let conn = test_conn();
        record_conn(&conn, &Signal::new(remote(433_920_000), 1)).unwrap();
        record_conn(&conn, &Signal::new(tag("AAAA"), 1)).unwrap();

        clear_conn(&conn).unwrap();
        assert!(counts_by_chip_conn(&conn).unwrap().is_empty());
        assert!(list_by_chip_conn(&conn, Chip::SubGhz).unwrap().is_empty());
    }

    #[test]
    fn get_returns_none_for_an_unknown_id() {
        let conn = test_conn();
        assert!(get_conn(&conn, "nfc-0000000000000000").unwrap().is_none());
    }
}
