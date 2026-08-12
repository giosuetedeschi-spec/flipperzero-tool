//! A fake Flipper that speaks the real RPC protocol.
//!
//! Without hardware there is no other way to exercise sessions, framing and
//! commands end to end. This serves a synthetic SD card over exactly the wire
//! format a real device uses, so everything above [`crate::transport::Transport`]
//! runs unchanged against it.
//!
//! The request handling is a pure function over `RpcMessage`, deliberately: it
//! can be tested without opening a socket, and the networking around it stays
//! thin enough to be obviously correct.

use crate::errors::AppError;
use crate::proto_bus::{FileInfoProto, RpcContent, RpcMessage};
use std::collections::BTreeMap;

/// The fake SD card.
///
/// A `BTreeMap` so listings come out in a stable order -- a test that depends on
/// hash iteration order passes and fails at random.
#[derive(Debug, Clone)]
pub struct MockFilesystem {
    files: BTreeMap<String, Vec<u8>>,
}

impl Default for MockFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFilesystem {
    /// A card populated the way a used Flipper's would be, including a file per
    /// format the app can parse.
    pub fn new() -> Self {
        let mut files = BTreeMap::new();

        files.insert(
            "/ext/subghz/gate.sub".to_string(),
            b"Filetype: Flipper SubGhz Key File\nVersion: 1\nFrequency: 433920000\n\
              Preset: FuriHalSubGhzPresetOok650Async\nProtocol: Princeton\nBit: 24\n\
              Key: 00 00 00 00 00 5C A8 E4\n"
                .to_vec(),
        );
        files.insert(
            "/ext/nfc/office_badge.nfc".to_string(),
            b"Filetype: Flipper NFC device\nVersion: 4\nDevice type: UID\n\
              UID: 04 1E 23 4A 5B 6C\nATQA: 00 44\nSAK: 08\n"
                .to_vec(),
        );
        files.insert(
            "/ext/lfrfid/door.rfid".to_string(),
            b"Filetype: Flipper RFID key\nVersion: 1\nKey type: EM4100\nData: 12 34 56 78 90\n"
                .to_vec(),
        );
        files.insert(
            "/ext/ibutton/intercom.ibtn".to_string(),
            b"Filetype: Flipper iButton key\nVersion: 1\nKey type: Dallas\n\
              Data: 01 02 03 04 05 06 07 08\n"
                .to_vec(),
        );
        files.insert(
            "/ext/infrared/tv.ir".to_string(),
            b"Filetype: IR signals file\nVersion: 1\nname: Power\ntype: parsed\n\
              protocol: NEC\naddress: 04 00 00 00\ncommand: 08 00 00 00\n"
                .to_vec(),
        );
        // Deliberately not valid UTF-8: this is what proves the binary path
        // works, which the old CLI transport could never do.
        files.insert(
            "/ext/apps/Tools/signal_radar.fap".to_string(),
            vec![0x7F, b'E', b'L', b'F', 0x00, 0xFF, 0xFE, 0x80],
        );

        Self { files }
    }

    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    /// Entries directly inside `path`, files and directories alike.
    ///
    /// Directories are inferred from the file paths rather than stored, which
    /// keeps the fixture above to a list of files.
    pub fn list(&self, path: &str) -> Vec<FileInfoProto> {
        let prefix = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        let mut directories: BTreeMap<String, ()> = BTreeMap::new();
        let mut entries = Vec::new();

        for (full_path, contents) in &self.files {
            let Some(remainder) = full_path.strip_prefix(&prefix) else {
                continue;
            };
            match remainder.split_once('/') {
                // Something deeper: record only the directory it sits in.
                Some((directory, _)) => {
                    directories.insert(directory.to_string(), ());
                }
                None => entries.push(FileInfoProto {
                    name: remainder.to_string(),
                    size: contents.len() as u32,
                    is_dir: false,
                }),
            }
        }

        let mut out: Vec<FileInfoProto> = directories
            .into_keys()
            .map(|name| FileInfoProto {
                name,
                size: 0,
                is_dir: true,
            })
            .collect();
        out.extend(entries);
        out
    }

    pub fn read(&self, path: &str) -> Option<&Vec<u8>> {
        self.files.get(path)
    }

    pub fn write(&mut self, path: &str, data: Vec<u8>) {
        self.files.insert(path.to_string(), data);
    }

    pub fn delete(&mut self, path: &str) -> bool {
        self.files.remove(path).is_some()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// The fake device: a filesystem plus the identity it reports.
#[derive(Debug, Clone)]
pub struct MockDevice {
    pub filesystem: MockFilesystem,
    pub name: String,
    pub model: String,
    pub firmware: String,
}

impl Default for MockDevice {
    fn default() -> Self {
        Self {
            filesystem: MockFilesystem::new(),
            name: "MockFlipper".to_string(),
            model: "Flipper Zero".to_string(),
            firmware: "1.0.0".to_string(),
        }
    }
}

impl MockDevice {
    /// Answer one request.
    ///
    /// Returns `None` for `StopSession`, which has no reply and ends the
    /// conversation. The `sequence_id` is echoed back: a mock that ignored it
    /// would let a client with broken correlation pass its tests.
    pub fn handle(&mut self, request: &RpcMessage) -> Option<RpcMessage> {
        let sequence_id = request.sequence_id;

        let content = match request.content.as_ref()? {
            RpcContent::StorageListRequest { path } => Some(RpcContent::StorageListResponse {
                files: self.filesystem.list(path),
            }),
            RpcContent::StorageReadRequest { path } => match self.filesystem.read(path) {
                Some(data) => Some(RpcContent::StorageReadResponse { data: data.clone() }),
                // A real device answers a missing path with an error status.
                // Reporting success with no data would let a client that never
                // checks for absence look correct.
                None => Some(RpcContent::PingResponse {
                    data: format!("not found: {}", path).into_bytes(),
                }),
            },
            RpcContent::StorageWriteRequest { path, data } => {
                self.filesystem.write(path, data.clone());
                Some(RpcContent::StorageWriteResponse)
            }
            RpcContent::StorageDeleteRequest { path } => {
                self.filesystem.delete(path);
                Some(RpcContent::StorageDeleteResponse)
            }
            RpcContent::StorageMkdirRequest { .. } => Some(RpcContent::StorageMkdirResponse),
            RpcContent::DeviceInfoRequest => Some(RpcContent::DeviceInfoResponse {
                name: self.name.clone(),
                model: self.model.clone(),
                firmware: self.firmware.clone(),
            }),
            RpcContent::PingRequest { data } => {
                Some(RpcContent::PingResponse { data: data.clone() })
            }
            RpcContent::StopSession => None,
            // Responses arriving as requests mean the client is confused; say so
            // rather than answering something plausible.
            _ => Some(RpcContent::PingResponse {
                data: b"unsupported request".to_vec(),
            }),
        }?;

        Some(RpcMessage {
            sequence_id,
            content: Some(content),
        })
    }

    /// Decode a frame, handle it, and encode the reply.
    pub fn handle_frame(&mut self, frame: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        let request = RpcMessage::from_bytes(frame)?;
        Ok(self.handle(&request).map(|reply| reply.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(sequence_id: u32, content: RpcContent) -> RpcMessage {
        RpcMessage {
            sequence_id,
            content: Some(content),
        }
    }

    #[test]
    fn lists_only_the_entries_directly_inside_a_directory() {
        let device = MockDevice::default();
        let entries = device.filesystem.list("/ext");

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"subghz"), "got {:?}", names);
        assert!(names.contains(&"nfc"));
        // gate.sub lives one level deeper and must not surface here.
        assert!(!names.contains(&"gate.sub"));
        assert!(
            entries
                .iter()
                .filter(|e| e.name == "subghz")
                .all(|e| e.is_dir)
        );
    }

    #[test]
    fn lists_files_with_their_real_size() {
        let device = MockDevice::default();
        let entries = device.filesystem.list("/ext/subghz");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "gate.sub");
        assert!(!entries[0].is_dir);
        assert!(entries[0].size > 0);
    }

    #[test]
    fn listing_an_unknown_directory_is_empty_rather_than_an_error() {
        let device = MockDevice::default();
        assert!(device.filesystem.list("/ext/nowhere").is_empty());
    }

    #[test]
    fn echoes_the_sequence_id() {
        // A mock that ignored this would let a client with broken correlation
        // pass every test it has.
        let mut device = MockDevice::default();
        let reply = device
            .handle(&request(4242, RpcContent::DeviceInfoRequest))
            .expect("device info is always answered");
        assert_eq!(reply.sequence_id, 4242);
    }

    #[test]
    fn reports_its_identity() {
        let mut device = MockDevice::default();
        let reply = device
            .handle(&request(1, RpcContent::DeviceInfoRequest))
            .unwrap();
        match reply.content {
            Some(RpcContent::DeviceInfoResponse { model, .. }) => assert_eq!(model, "Flipper Zero"),
            other => panic!("expected device info, got {:?}", other),
        }
    }

    #[test]
    fn serves_a_file_that_is_not_valid_utf8() {
        // The case the old CLI transport could never handle.
        let mut device = MockDevice::default();
        let reply = device
            .handle(&request(
                1,
                RpcContent::StorageReadRequest {
                    path: "/ext/apps/Tools/signal_radar.fap".to_string(),
                },
            ))
            .unwrap();

        match reply.content {
            Some(RpcContent::StorageReadResponse { data }) => {
                assert_eq!(&data[..4], b"\x7FELF");
                assert!(String::from_utf8(data).is_err(), "fixture must be binary");
            }
            other => panic!("expected a read response, got {:?}", other),
        }
    }

    #[test]
    fn a_missing_file_is_reported_rather_than_answered_with_nothing() {
        let mut device = MockDevice::default();
        let reply = device
            .handle(&request(
                1,
                RpcContent::StorageReadRequest {
                    path: "/ext/does/not/exist".to_string(),
                },
            ))
            .unwrap();

        match reply.content {
            Some(RpcContent::PingResponse { data }) => {
                assert!(String::from_utf8_lossy(&data).contains("not found"));
            }
            other => panic!("expected an error reply, got {:?}", other),
        }
    }

    #[test]
    fn a_written_file_can_be_read_back_byte_for_byte() {
        let mut device = MockDevice::default();
        let payload: Vec<u8> = (0u8..=255).collect();

        device
            .handle(&request(
                1,
                RpcContent::StorageWriteRequest {
                    path: "/ext/apps/Tools/new.fap".to_string(),
                    data: payload.clone(),
                },
            ))
            .unwrap();

        let reply = device
            .handle(&request(
                2,
                RpcContent::StorageReadRequest {
                    path: "/ext/apps/Tools/new.fap".to_string(),
                },
            ))
            .unwrap();

        match reply.content {
            Some(RpcContent::StorageReadResponse { data }) => assert_eq!(data, payload),
            other => panic!("expected a read response, got {:?}", other),
        }
    }

    #[test]
    fn deleting_removes_the_file() {
        let mut device = MockDevice::default();
        let before = device.filesystem.file_count();

        device
            .handle(&request(
                1,
                RpcContent::StorageDeleteRequest {
                    path: "/ext/subghz/gate.sub".to_string(),
                },
            ))
            .unwrap();

        assert_eq!(device.filesystem.file_count(), before - 1);
        assert!(device.filesystem.read("/ext/subghz/gate.sub").is_none());
    }

    #[test]
    fn ping_echoes_its_payload() {
        let mut device = MockDevice::default();
        let reply = device
            .handle(&request(
                7,
                RpcContent::PingRequest {
                    data: b"are you there".to_vec(),
                },
            ))
            .unwrap();

        match reply.content {
            Some(RpcContent::PingResponse { data }) => assert_eq!(data, b"are you there"),
            other => panic!("expected a pong, got {:?}", other),
        }
    }

    #[test]
    fn stop_session_ends_the_conversation_without_a_reply() {
        let mut device = MockDevice::default();
        assert!(
            device
                .handle(&request(1, RpcContent::StopSession))
                .is_none()
        );
    }

    #[test]
    fn frames_round_trip_through_the_codec() {
        let mut device = MockDevice::default();
        let encoded = request(9, RpcContent::DeviceInfoRequest).to_bytes();

        let reply_bytes = device.handle_frame(&encoded).unwrap().expect("a reply");
        let reply = RpcMessage::from_bytes(&reply_bytes).unwrap();
        assert_eq!(reply.sequence_id, 9);
    }

    #[test]
    fn an_empty_filesystem_serves_nothing() {
        let mut device = MockDevice {
            filesystem: MockFilesystem::empty(),
            ..Default::default()
        };
        assert!(device.filesystem.list("/ext").is_empty());
        assert!(
            device
                .handle(&request(
                    1,
                    RpcContent::StorageReadRequest {
                        path: "/ext/subghz/gate.sub".to_string()
                    }
                ))
                .is_some(),
            "a missing file still gets an answer"
        );
    }
}
