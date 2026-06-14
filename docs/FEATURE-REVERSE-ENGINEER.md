# Feature: Reverse Engineer

## Stato: COMPLETATO

## Descrizione

Modulo di reverse engineering per analizzare dati grezzi (byte streams) e identificare
pattern, protocolli e struttura. Utile per analizzare segnali sconosciuti acquisiti
dal Flipper Zero.

## Funzionalita

1. **Analisi Entropia**: calcola entropia Shannon (0-8 bit) per distinguere
   plaintext da dati compressi/criptati
2. **Pattern Detection**: identifica sequenze ripetibili (preamble, sync word)
3. **Protocol Fingerprinting**: confronta con firme di protocolli noti
   (NEC IR, RC5, RC6, Sony SIRC, Sub-GHz OOK, NFC NDEF, Protobuf)
4. **Structure Inference**: identifica header, length field, payload, checksum
5. **Hex/ASCII Preview**: visualizzazione rapida dei dati

## Architettura

### Backend: reverse_engineer.rs

Modulo Rust con le seguenti funzioni:

- `calculate_entropy(data: &[u8]) -> f64` — entropia Shannon
- `find_patterns(data, min_len, max_len) -> Vec<PatternMatch>` — pattern ripetitivi
- `fingerprint_protocols(data) -> Vec<ProtocolFingerprint>` — confronto firme
- `infer_structure(data) -> Vec<FieldCandidate>` — inferenza struttura
- `analyze(data) -> AnalysisResult` — analisi completa

Comandi Tauri:
- `reverse_engineer_analyze(hex_data: String)` — analizza hex string
- `reverse_engineer_analyze_file(path: String)` — analizza file su disco

### Frontend: ReverseEngineerPanel.tsx

Pannello laterale con:
- Tab "Hex Input": incolla hex string e analizza
- Tab "From File": analizza un file dal filesystem
- Risultati: overview, protocolli trovati, pattern, struttura inferita
- Colori per confidenza (verde > 70%, giallo > 40%, rosso < 40%)

### Integrazione: App.tsx

Bottone "Reverse Engineer" nell'header per mostrare/nascondere il pannello.

## File Creati/Modificati

| File | Azione |
|------|--------|
| src-tauri/src/reverse_engineer.rs | CREATO - modulo analisi |
| src-tauri/src/lib.rs | MODIFICATO - aggiunto mod re-export |
| frontend/src/components/ReverseEngineerPanel.tsx | CREATO - pannello UI |
| frontend/src/services/tauri.ts | MODIFICATO - aggiunte funzioni invoke |
| frontend/src/App.tsx | MODIFICATO - aggiunto bottone + pannello |

## Protocolli Riconosciuti

| Protocollo | Signature | Descrizione |
|-----------|-----------|-------------|
| NEC IR | 00 FF | Infrared NEC 32-bit |
| NEC repeat | 00 FF 00 00 | Codice ripetizione NEC |
| RC5 | 36 | Protocollo RC5 |
| RC6 | 37 | Protocollo RC6 |
| Sony SIRC | 01 | Sony SIRC 12-bit |
| Sub-GHz OOK | AA AA | Preamble OOK |
| NFC NDEF | 03 | Messaggio NDEF |
| Protobuf varint | (pattern) | Encoding varint rilevato |

## Testing

1. Apri il pannello "Reverse Engineer"
2. Incolla hex data es: `AABBCCDDEEFF00112233`
3. Clicca "Analyze"
4. Verifica: entropia, pattern, protocolli, struttura

## Esempio Output

Input: `00FF1122334455667788`
- Entropia: 3.12 (plaintext)
- Protocolli: NEC IR (00 FF)
- Pattern: nessun pattern ripetitivo
- Structure: Header 00FF, Length 11, Payload 22334455667788
