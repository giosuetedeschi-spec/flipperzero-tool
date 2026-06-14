# Feature: ProtoBus

## Stato: COMPLETATO

## Descrizione

Layer Protobuf per comunicazione con Flipper Zero via seriale.
Fornisce encode/decode messaggi RPC secondo il protocollo flipper.proto.

## Funzionalita

1. **Encode/Decode Protobuf**: serializzazione manuale wire-format compatibile
2. **Varint Encoding**: LEB128 per tutti i tipi varint
3. **Message Framing**: length-prefixed con varint
4. **RPC Types**: tutti i messaggi da flipper.proto (StorageList, Read, Write, Delete, Mkdir, DeviceInfo, Ping, Stop)
5. **Session Management**: sequence ID tracking

## Architettura

### Modulo: proto_bus.rs

Tipi:
- RpcMessage { sequence_id, content: Option<RpcContent> }
- RpcContent enum (tutti i tipi di messaggio)
- FileInfoProto { name, size, is_dir }
- SessionState { session_id, next_sequence, pending_requests }

Funzioni encode/decode:
- RpcMessage::to_bytes() -> Vec<u8>
- RpcMessage::from_bytes(&[u8]) -> Result<RpcMessage>
- encode_varint/decode_varint (LEB128)
- encode_string_field, encode_bytes_field, encode_message_field

Funzioni RPC alto livello:
- rpc_command(state, content) -> Result<RpcMessage>
- proto_list_dir(state, path) -> Result<Vec<FileInfoProto>>
- proto_read_file(state, path) -> Result<Vec<u8>>
- proto_write_file(state, path, data) -> Result<()>
- proto_mkdir(state, path) -> Result<()>
- proto_delete(state, path) -> Result<()>
- proto_device_info(state) -> Result<(name, model, firmware)>
- proto_ping(state, data) -> Result<Vec<u8>>

### Modulo: serial.rs (esteso)

Aggiunto a FlipperConnection:
- next_seq: u32 field
- next_sequence() / current_sequence() methods
- write_rpc(data) - scrive messaggio length-prefixed
- read_rpc(timeout_ms) - legge messaggio length-prefixed

## File Creati/Modificati

| File | Azione |
|------|--------|
| src-tauri/src/proto_bus.rs | CREATO - layer protobuf RPC |
| src-tauri/src/serial.rs | MODIFICATO - aggiunto next_seq, write_rpc, read_rpc |
| src-tauri/src/lib.rs | MODIFICATO - aggiunto mod proto_bus + re-exports |

## Stato Serial Integration

L'integrazione seriale completa (write_rpc/read_rpc che parlano effettivamente
con la porta seriale) richiede che FlipperConnection abbia accesso al SerialPort.
Attualmente FlipperConnection non ha il campo port - e' in FlipperState.
La prossima fase sara' spostare il port handle in FlipperConnection o creare
un metodo di FlipperState che delega a FlipperConnection.

Per ora, encode/decode funzionano correttamente per dati offline.
