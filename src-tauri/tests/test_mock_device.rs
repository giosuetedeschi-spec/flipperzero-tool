//! End-to-end tests against the mock Flipper over a real TCP socket.
//!
//! Everything else in the suite exercises one layer at a time. These drive the
//! whole stack the way the app does -- transport, framing, session, RPC codec,
//! and the parsers on top -- against a server in another thread. Without
//! hardware this is the only test that proves the layers fit together.

use flipperzero_tool_lib::mock_device::MockDevice;
use flipperzero_tool_lib::parsers::{ParsedFile, parse_rfid_struct};
use flipperzero_tool_lib::proto_bus::{
    RpcContent, proto_list_dir, proto_read_file, proto_write_file,
};
use flipperzero_tool_lib::rpc::FlipperSession;
use flipperzero_tool_lib::rpc::framing::{FrameReader, write_frame};
use flipperzero_tool_lib::transport::tcp::TcpTransport;
use flipperzero_tool_lib::transport::{Transport, TransportKind};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Start a mock on an ephemeral port and return its address.
///
/// Port 0 lets the OS choose, so tests never collide with each other or with a
/// developer's own running mock.
fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let address = listener
        .local_addr()
        .expect("read back the port")
        .to_string();

    thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        serve(stream);
    });

    address
}

/// Mirrors the loop in the `flipper_mock` binary.
fn serve(mut stream: TcpStream) {
    let mut device = MockDevice::default();
    let mut reader = FrameReader::new();
    let mut scratch = [0u8; 4096];

    loop {
        let read = match stream.read(&mut scratch) {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        reader.push(&scratch[..read]);

        while let Ok(Some(frame)) = reader.next_frame() {
            match device.handle_frame(&frame) {
                Ok(Some(reply)) => {
                    let mut sink = SocketSink {
                        stream: &mut stream,
                    };
                    if write_frame(&mut sink, &reply).is_err() {
                        return;
                    }
                }
                Ok(None) => return,
                Err(_) => return,
            }
        }
    }
}

struct SocketSink<'a> {
    stream: &'a mut TcpStream,
}

impl Transport for SocketSink<'_> {
    fn kind(&self) -> TransportKind {
        TransportKind::Loopback
    }
    fn write(&mut self, data: &[u8]) -> Result<(), flipperzero_tool_lib::AppError> {
        self.stream
            .write_all(data)
            .map_err(|e| flipperzero_tool_lib::AppError::SerialError(e.to_string()))
    }
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, flipperzero_tool_lib::AppError> {
        Ok(0)
    }
}

fn connect() -> FlipperSession {
    let address = spawn_mock();
    let transport = TcpTransport::connect(&address).expect("reach the mock");
    FlipperSession::new(Box::new(transport))
}

#[test]
fn lists_the_root_of_the_fake_sd_card() {
    let mut session = connect();
    let entries = proto_list_dir(&mut session, "/ext").expect("list /ext");

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"subghz"), "got {:?}", names);
    assert!(names.contains(&"nfc"), "got {:?}", names);
}

#[test]
fn reads_a_capture_and_parses_it() {
    // The full journey: socket, framing, RPC, then the parser that the UI uses.
    let mut session = connect();
    let bytes = proto_read_file(&mut session, "/ext/subghz/gate.sub").expect("read the capture");

    let text = String::from_utf8(bytes).expect("a .sub file is text");
    let parsed = ParsedFile::parse_sub_struct(&text).expect("parse it");

    assert_eq!(parsed.frequency, 433_920_000);
    assert_eq!(parsed.protocol, "Princeton");
}

#[test]
fn reads_one_of_the_formats_that_had_no_parser_before() {
    let mut session = connect();
    let bytes = proto_read_file(&mut session, "/ext/lfrfid/door.rfid").expect("read the badge");

    let parsed = parse_rfid_struct(&String::from_utf8(bytes).unwrap()).expect("parse it");
    assert_eq!(parsed.key_type, "EM4100");
    assert_eq!(parsed.data, "12 34 56 78 90");
}

#[test]
fn a_binary_file_survives_the_whole_stack() {
    // The capability the old CLI transport never had, proved end to end rather
    // than at the codec level.
    let mut session = connect();
    let bytes =
        proto_read_file(&mut session, "/ext/apps/Tools/signal_radar.fap").expect("read the fap");

    assert_eq!(&bytes[..4], b"\x7FELF");
    assert!(String::from_utf8(bytes).is_err(), "the payload is binary");
}

#[test]
fn a_written_file_comes_back_unchanged() {
    // What deploying the Signal Radar FAP from a phone depends on.
    let mut session = connect();
    let payload: Vec<u8> = (0u8..=255).cycle().take(1000).collect();

    proto_write_file(&mut session, "/ext/apps/Tools/deployed.fap", &payload).expect("write");
    let read_back = proto_read_file(&mut session, "/ext/apps/Tools/deployed.fap").expect("read");

    assert_eq!(read_back, payload);
}

#[test]
fn many_commands_stay_correlated_over_one_session() {
    // A payload well past one TCP segment, repeated, is where a session that
    // mismatched replies would come apart.
    let mut session = connect();

    for index in 0..12u8 {
        let path = format!("/ext/test/file{}.bin", index);
        let payload = vec![index; 700];

        proto_write_file(&mut session, &path, &payload).expect("write");
        assert_eq!(
            proto_read_file(&mut session, &path).expect("read"),
            payload,
            "reply {} did not match its request",
            index
        );
    }
}

#[test]
fn reports_the_device_identity() {
    let mut session = connect();
    let reply =
        flipperzero_tool_lib::proto_bus::rpc_command(&mut session, RpcContent::DeviceInfoRequest)
            .expect("device info");

    match reply.content {
        Some(RpcContent::DeviceInfoResponse { model, name, .. }) => {
            assert_eq!(model, "Flipper Zero");
            assert_eq!(name, "MockFlipper");
        }
        other => panic!("expected device info, got {:?}", other),
    }
}

#[test]
fn a_write_is_visible_in_the_next_listing() {
    let mut session = connect();
    proto_write_file(&mut session, "/ext/subghz/new_remote.sub", b"Filetype: x\n").expect("write");

    let entries = proto_list_dir(&mut session, "/ext/subghz").expect("list");
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"new_remote.sub"), "got {:?}", names);
}

#[test]
fn connecting_to_a_closed_port_fails_rather_than_hanging() {
    // Port 1 on loopback has nothing listening.
    assert!(TcpTransport::connect("127.0.0.1:1").is_err());
}

#[test]
fn a_server_that_hangs_up_produces_a_timeout_not_a_hang() {
    // A closed socket reads as "no bytes", which is indistinguishable from an
    // idle link at the byte level. Without the session's idle budget the client
    // would wait on a peer that is never coming back.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().unwrap().to_string();
    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            drop(stream); // accept, then hang up without answering
        }
    });

    let transport = TcpTransport::connect(&address).expect("connect");
    let mut session = FlipperSession::new(Box::new(transport)).with_max_idle_polls(3);

    let result =
        flipperzero_tool_lib::proto_bus::rpc_command(&mut session, RpcContent::DeviceInfoRequest);
    assert!(
        result.is_err(),
        "a dead peer must not block the caller forever"
    );
}
