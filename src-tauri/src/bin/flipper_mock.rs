//! A fake Flipper on a TCP socket.
//!
//! Run it, point the app at it, and the whole stack -- transport, framing,
//! session, RPC, parsers -- runs against something that answers exactly as the
//! device does, with no hardware in the room.
//!
//!     cargo run --bin flipper_mock          # listens on 127.0.0.1:9999
//!     cargo run --bin flipper_mock -- 4321  # or a port of your choosing
//!
//! Development only. Nothing ships a Flipper reachable over TCP.

use flipperzero_tool_lib::mock_device::MockDevice;
use flipperzero_tool_lib::rpc::framing::{FrameReader, write_frame};
use flipperzero_tool_lib::transport::Transport;
use flipperzero_tool_lib::transport::tcp::TcpTransport;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const DEFAULT_PORT: u16 = 9999;

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Cannot listen on 127.0.0.1:{}: {}", port, e);
            std::process::exit(1);
        }
    };

    println!("Mock Flipper listening on 127.0.0.1:{}", port);
    println!(
        "Point the app at it with TcpTransport::connect(\"127.0.0.1:{}\")",
        port
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // One client at a time, matching the device: a Flipper serves a
                // single RPC session, and letting the mock be more permissive
                // would hide concurrency bugs rather than expose them.
                let peer = stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                println!("Client connected: {}", peer);
                serve(stream);
                println!("Client disconnected: {}", peer);
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

/// Serve one client until it disconnects or sends StopSession.
fn serve(mut stream: TcpStream) {
    let mut device = MockDevice::default();
    let mut reader = FrameReader::new();
    let mut scratch = [0u8; 4096];

    loop {
        let read = match stream.read(&mut scratch) {
            Ok(0) => return, // clean disconnect
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                return;
            }
        };
        reader.push(&scratch[..read]);

        // A single read can carry several frames, or half of one.
        loop {
            let frame = match reader.next_frame() {
                Ok(Some(frame)) => frame,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Framing error, dropping the client: {}", e);
                    return;
                }
            };

            match device.handle_frame(&frame) {
                Ok(Some(reply)) => {
                    let mut transport = StreamTransport {
                        stream: &mut stream,
                    };
                    if let Err(e) = write_frame(&mut transport, &reply) {
                        eprintln!("Write error: {}", e);
                        return;
                    }
                }
                // StopSession: the client is done and wants no answer.
                Ok(None) => return,
                Err(e) => eprintln!("Could not handle a request: {}", e),
            }
        }
    }
}

/// Lets the server reuse `write_frame`, so both ends frame identically.
///
/// Reusing the client's framing code on the server is deliberate: a mock with
/// its own encoder could drift from the real one and quietly validate a bug.
struct StreamTransport<'a> {
    stream: &'a mut TcpStream,
}

impl Transport for StreamTransport<'_> {
    fn kind(&self) -> flipperzero_tool_lib::transport::TransportKind {
        flipperzero_tool_lib::transport::TransportKind::Loopback
    }

    fn write(&mut self, data: &[u8]) -> Result<(), flipperzero_tool_lib::AppError> {
        self.stream
            .write_all(data)
            .map_err(|e| flipperzero_tool_lib::AppError::SerialError(format!("Write error: {}", e)))
    }

    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, flipperzero_tool_lib::AppError> {
        // The server never reads through this shim; it owns the socket directly.
        Ok(0)
    }
}

/// Keeps the unused-import warning honest: the binary genuinely offers
/// `TcpTransport` as the way to reach itself.
#[allow(dead_code)]
fn connect_hint(address: &str) -> Result<TcpTransport, flipperzero_tool_lib::AppError> {
    TcpTransport::connect(address)
}
