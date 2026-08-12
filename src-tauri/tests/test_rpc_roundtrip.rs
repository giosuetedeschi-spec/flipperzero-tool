//! End-to-end tests for the RPC path: encode a request, frame it, push it
//! through a transport, decode the reply.
//!
//! Until now `rpc_command` was a stub that always returned an error, so nothing
//! exercised the protobuf codec and the wire together. These tests stand a fake
//! device up behind a loopback transport and drive the real code path.

use flipperzero_tool_lib::errors::AppError;
use flipperzero_tool_lib::proto_bus::{
    FileInfoProto, RpcContent, RpcMessage, proto_list_dir, proto_ping, proto_read_file,
    proto_write_file, rpc_command,
};
use flipperzero_tool_lib::rpc::FlipperSession;
use flipperzero_tool_lib::serial::encode_varint;
use flipperzero_tool_lib::transport::loopback::{LoopbackBuffers, LoopbackTransport};
use std::sync::{Arc, Mutex};

/// Queue a message the fake device will "send", length-delimited like the real one.
fn queue_reply(buffers: &Arc<Mutex<LoopbackBuffers>>, message: RpcMessage) {
    let payload = message.to_bytes();
    let mut framed = Vec::new();
    encode_varint(&mut framed, payload.len() as u64);
    framed.extend_from_slice(&payload);
    buffers.lock().unwrap().from_device.extend(framed);
}

/// A session over a link that behaves like BLE: small writes, dribbled reads.
fn ble_shaped_session() -> (FlipperSession, Arc<Mutex<LoopbackBuffers>>) {
    let (transport, buffers) = LoopbackTransport::with_limits(20, 9);
    (FlipperSession::new(Box::new(transport)), buffers)
}

#[test]
fn list_dir_round_trips_through_the_wire() {
    let (mut session, buffers) = ble_shaped_session();
    queue_reply(
        &buffers,
        RpcMessage {
            // The first id a fresh session allocates.
            sequence_id: 1,
            content: Some(RpcContent::StorageListResponse {
                files: vec![
                    FileInfoProto {
                        name: "subghz".to_string(),
                        size: 0,
                        is_dir: true,
                    },
                    FileInfoProto {
                        name: "gate.sub".to_string(),
                        size: 512,
                        is_dir: false,
                    },
                ],
            }),
        },
    );

    let files = proto_list_dir(&mut session, "/ext").expect("list should succeed");

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "subghz");
    assert!(files[0].is_dir);
    assert_eq!(files[1].name, "gate.sub");
    assert_eq!(files[1].size, 512);
    assert!(!files[1].is_dir);
}

#[test]
fn read_file_returns_bytes_that_are_not_valid_utf8() {
    // The reason the RPC path exists: the old CLI path treated every payload as
    // text, so binary files could not survive the trip.
    let binary: Vec<u8> = vec![0x00, 0xFF, 0xFE, 0x80, 0x01, 0x7F];

    let (mut session, buffers) = ble_shaped_session();
    queue_reply(
        &buffers,
        RpcMessage {
            sequence_id: 1,
            content: Some(RpcContent::StorageReadResponse {
                data: binary.clone(),
            }),
        },
    );

    assert_eq!(
        proto_read_file(&mut session, "/ext/nfc/card.nfc").unwrap(),
        binary
    );
}

#[test]
fn write_file_sends_the_payload_and_accepts_the_ack() {
    let (mut session, buffers) = ble_shaped_session();
    queue_reply(
        &buffers,
        RpcMessage {
            sequence_id: 1,
            content: Some(RpcContent::StorageWriteResponse),
        },
    );

    proto_write_file(
        &mut session,
        "/ext/apps/Tools/radar.fap",
        &[0xDE, 0xAD, 0xBE, 0xEF],
    )
    .unwrap();

    // Something was actually put on the wire, chunked to the 20-byte limit.
    assert!(!buffers.lock().unwrap().to_device.is_empty());
}

#[test]
fn ping_round_trips() {
    let (mut session, buffers) = ble_shaped_session();
    queue_reply(
        &buffers,
        RpcMessage {
            sequence_id: 1,
            content: Some(RpcContent::PingResponse {
                data: b"alive".to_vec(),
            }),
        },
    );

    assert_eq!(proto_ping(&mut session, b"alive").unwrap(), b"alive");
}

#[test]
fn unsolicited_messages_do_not_desynchronise_the_stream() {
    // The Signal Radar FAP streams telemetry while commands are in flight. A
    // reply matcher that just took "the next frame" would return the telemetry
    // and then be one message behind forever.
    let (mut session, buffers) = ble_shaped_session();

    for _ in 0..3 {
        queue_reply(
            &buffers,
            RpcMessage {
                sequence_id: 0, // unsolicited: not an answer to anything
                content: Some(RpcContent::PingResponse {
                    data: b"telemetry".to_vec(),
                }),
            },
        );
    }
    queue_reply(
        &buffers,
        RpcMessage {
            sequence_id: 1,
            content: Some(RpcContent::PingResponse {
                data: b"the actual answer".to_vec(),
            }),
        },
    );

    assert_eq!(
        proto_ping(&mut session, b"hello").unwrap(),
        b"the actual answer"
    );
}

#[test]
fn sequence_ids_advance_across_commands() {
    let (mut session, buffers) = ble_shaped_session();
    for id in 1..=3 {
        queue_reply(
            &buffers,
            RpcMessage {
                sequence_id: id,
                content: Some(RpcContent::PingResponse {
                    data: vec![id as u8],
                }),
            },
        );
    }

    for expected in 1..=3u8 {
        assert_eq!(proto_ping(&mut session, b"x").unwrap(), vec![expected]);
    }
}

#[test]
fn a_wrong_response_type_is_reported_rather_than_silently_accepted() {
    let (mut session, buffers) = ble_shaped_session();
    queue_reply(
        &buffers,
        RpcMessage {
            sequence_id: 1,
            content: Some(RpcContent::PingResponse { data: vec![] }),
        },
    );

    let err = proto_list_dir(&mut session, "/ext").unwrap_err();
    assert!(matches!(err, AppError::SerialError(_)), "got {:?}", err);
}

#[test]
fn a_silent_device_produces_a_timeout_not_a_hang() {
    let (transport, _buffers) = LoopbackTransport::new();
    let mut session = FlipperSession::new(Box::new(transport)).with_max_idle_polls(2);

    let err = rpc_command(&mut session, RpcContent::DeviceInfoRequest).unwrap_err();
    assert!(matches!(err, AppError::SerialError(_)), "got {:?}", err);
}
