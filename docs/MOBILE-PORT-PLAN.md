# Porting FlipperZero Tool → iOS + Android (Tauri v2) + Signal Radar

## Context

`flipperzero-tool` oggi è un'app **desktop** Tauri v2 + React 19: gestore file per la SD del Flipper,
parser `.sub`/`.ir`/`.nfc`, editor CodeMirror, reverse engineering, wrapper uFBT. Non pilota il
dispositivo: non trasmette, non emula, non legge tag.

Obiettivo doppio:

1. **Portare l'app su iOS e Android** riusando lo stesso codebase (Tauri v2 supporta entrambi), con
   **parità di feature 1:1 col desktop**.
2. **Aggiungere la feature di punta, il "Signal Radar"**: uno schema interattivo del Flipper con tutti
   i suoi chip; toccando un chip l'utente vede *che tipo di segnale è*, *quanti ne sono stati visti*, e
   *cosa può farci*. L'app interroga il Flipper a ritmo costante su tutti i chip e presenta i risultati
   in modo ordinato, con spiegazioni in linguaggio semplice per chi non sa nulla di RF.

Due ostacoli strutturali determinano l'intera architettura:

- **`serialport` non compila su iOS/Android.** L'unico layer dispositivo (`src-tauri/src/serial/mod.rs`,
  498 LOC) è USB CDC desktop-only e fa *prompt-scraping* del CLI testuale riaprendo la porta a ogni
  comando — `FlipperConnection` (`serial/mod.rs:101`) non conserva nessun handle.
- **Il firmware stock non espone via RPC nessuno scan continuo** di SubGHz/NFC/RFID/IR. L'RPC dà solo
  storage, sistema, GUI, avvio app, GPIO. Il flusso live richiede un **FAP custom** sul Flipper.

## Decisioni prese

| Tema | Scelta |
|---|---|
| Framework | Tauri v2 mobile, stesso repo, stesso crate `flipperzero_tool_lib` |
| Scope | Port 1:1 del desktop **+** vista Radar |
| Trasporto | BLE su iOS+Android, **+** USB-OTG su Android, USB CDC su desktop |
| Live data | FAP custom sul Flipper, con fallback alla libreria dei file salvati |
| Firmware | OFW primario; custom firmware (Momentum/Unleashed/RogueMaster) rilevato → feature extra |
| Chip | SubGHz, NFC, LF RFID 125 kHz, iButton, IR, BLE, GPIO **+ moduli esterni** (devboard WiFi/ESP32, CC1101 esterno, NRF24) |
| Azioni | Replay/emulazione/TX con gate legale, analisi/decodifica avanzata, gestione libreria |
| Polling | Adattivo **+ scan in background** con notifiche push |
| Storico | SQLite persistente **+ geolocalizzazione** dei rilevamenti |
| Lingua | Bilingue IT/EN, i18n dal giorno 1 |
| Distribuzione | Sideload / uso personale (APK diretto, TestFlight/dev-build su iOS) |
| Design | Dark coerente col desktop, ma linguaggio semplice, icone grandi, onboarding, tooltip ovunque |
| Test | Flipper fisico **+ mock/emulatore** per sviluppo quotidiano e CI |
| Merge | Push su `claude/flipper-ios-android-port-bgbt4x` → PR → merge su `main` |

---

## Architettura

### A. Trasporto — il perno di tutto

Nuovo modulo `src-tauri/src/transport/` che astrae il byte-stream, così desktop/Android/iOS
condividono un unico percorso di codice:

```
transport/mod.rs      trait Transport { write/read async, mtu(), kind() }  + TransportKind enum
transport/usb_cdc.rs  #[cfg(desktop)]  — codice esistente di serial/mod.rs, spostato qui
transport/ble.rs      tutte le piattaforme — GATT Flipper Serial Service
                      19ed82ae-ed21-4c9d-4145-228e61fe0000 (RX/TX/flow-control chars)
transport/usb_otg.rs  #[cfg(target_os="android")] — plugin Kotlin su UsbManager
transport/mock.rs     TCP verso l'emulatore locale (dev + CI)
```

Sopra, nuovo modulo `src-tauri/src/rpc/`:

- `session.rs` — **`FlipperSession` possiede la connessione persistente** (elimina il difetto
  riapri-a-ogni-comando). Task lettore in background, framing protobuf a **varint length-prefix**,
  correlazione richiesta/risposta per `command_id` con `tokio::sync::oneshot`, riassemblaggio delle
  risposte `has_next`, `tokio::sync::broadcast` per gli eventi non sollecitati (telemetria del FAP).
- `chunking.rs` — segmentazione sull'MTU BLE negoziato + flow control (il Flipper ha una
  characteristic di flow control: va rispettata o si perdono pacchetti).

**Scelta della libreria BLE — punto di rischio numero 1.** `btleplug` copre desktop+iOS+Android ma su
Android richiede un bridge JNI che in Tauri v2 è fragile. Percorso consigliato: partire da
`tauri-plugin-blec` (plugin community su btleplug), e se blocca su MTU/background/flow-control,
scrivere un **plugin Tauri mobile nostro** (`src-tauri/plugins/flipper-ble/` con
`ios/Sources/*.swift` su CoreBluetooth e `android/src/main/kotlin/*.kt` su `BluetoothGatt`).
Il piano deve essere eseguito assumendo che il fallback al plugin custom sia probabile, non
eccezionale.

Per **USB-OTG Android** non esiste plugin Tauri: serve un plugin Kotlin che faccia
`UsbManager.requestPermission` → `UsbDeviceConnection` → CDC bulk in/out sugli endpoint del
VID `0x0483` / PID `0x5740` (già costanti in `serial/mod.rs:19`).

Il vecchio percorso CLI text (`storage list`, scraping di `">:"`) resta solo come fallback desktop e
va marcato deprecato: su BLE è inadeguato.

### B. Protobuf — sostituire il codec scritto a mano

`src-tauri/proto/flipper.proto` è un sottoinsieme scritto a mano, **non compilato** da `build.rs`, già
divergente dall'upstream (`DeviceInfoResponse` ha forme diverse tra `.proto` e Rust). Mantenerlo a mano
per l'intera superficie RPC (System/Storage/Gui/App/Gpio) non è sostenibile.

**Decisione:** vendorare l'upstream `flipperzero-protobuf` a commit fissato in
`src-tauri/proto/vendor/` e generare con `prost-build` + `protoc-bin-vendored` in `build.rs` (protoc
non richiesto sulla macchina né in CI). Del `proto_bus.rs` esistente si conservano solo gli helper
varint e i loro test; il codec di messaggi scritto a mano viene ritirato.
Nota storica da tenere presente: il CHANGELOG dice che `prost`/`protoc` erano già stati **rimossi** in
passato — se il codegen ridà problemi, il piano B è `micropb`, non tornare al codec manuale.

### C. Il FAP custom — `flipper-fap/`

Nuovo sotto-progetto C compilato con uFBT: `flipper-fap/signal_radar/`.

- **Cosa fa:** ciclo di scansione per chip — SubGHz (hopping di frequenza + RSSI + decodifica
  protocollo), NFC (poller ISO14443A/B, ISO15693, FeliCa → UID/tipo), LF RFID (EM4100/HID), iButton
  (1-Wire), IR (ricezione + decodifica), GPIO (probe UART/SPI per riconoscere devboard ESP32,
  CC1101 esterno, NRF24).
- **Canale di ritorno:** RPC `App.DataExchange` (blob binari bidirezionali) — il mobile manda comandi
  compatti, il FAP risponde con eventi. Formato: struct binaria compatta versionata (non JSON: la
  banda BLE è ~1–8 KB/s).
- **Versionamento e deploy:** il `.fap` è impacchettato come risorsa Tauri, scritto su
  `/ext/apps/Tools/signal_radar.fap` via `Storage.Write` (richiede il fix del round-trip binario) e
  avviato con `App.StartRequest`. Handshake di versione all'avvio; mismatch → ri-deploy automatico.
- **Matrice firmware:** build del FAP contro SDK OFW + Momentum + Unleashed; l'app sceglie il binario
  giusto dall'`api_version` letto da `System.DeviceInfo`.
- **Fallback:** FAP assente o non avviabile → modalità libreria sui file `.sub/.nfc/.rfid/.ir/.ibtn`
  già salvati su SD, con la stessa UI Radar ma marcata "storico, non live".

**Limite da comunicare all'utente nell'app:** mentre il telefono è connesso via BLE, la radio BLE del
Flipper è occupata a servire quella connessione — lo **scan BLE dell'ambiente circostante non è
realmente disponibile in quella modalità**. Funziona su Android via USB-OTG. La UI deve mostrare il
chip Bluetooth come "non disponibile in questa modalità" invece di fingere dati.

### D. Dominio: modello `Signal` unificato

`src-tauri/src/signals/`:

- `model.rs` — `Signal { id, chip: Chip, kind: SignalKind, first_seen, last_seen, count, rssi,
  location: Option<GeoPoint>, raw, actions: Vec<ActionId>, explain_key }`. `Chip` copre
  SubGhz/Nfc/LfRfid/IButton/Infrared/Ble/Gpio/External(kind). `id` = hash stabile della fingerprint,
  così i rilevamenti ripetuti incrementano `count` invece di creare duplicati (è ciò che risponde a
  "quanti ce ne sono").
- `store.rs` — tabelle `signals` e `signal_sightings(signal_id, ts, rssi, lat, lon)` nello stesso
  SQLite già inizializzato da `src-tauri/src/vfs.rs` (riusare `vfs::init_cache`, non aprire un secondo DB).
- `actions.rs` — catalogo `ActionId` → cosa si può fare per tipo di segnale (salva, replay, emula,
  analizza, esporta, confronta), con flag `requires_tx` che attiva il gate legale.
- `explain.rs` — solo **chiavi** di spiegazione (`explain_key`); il testo vive nei file i18n TS, così
  si traduce e si riscrive senza ricompilare Rust.

I parser esistenti (`parsers.rs`) e `reverse_engineer.rs` vengono riusati come *arricchitori* del
`Signal`, non riscritti. Vanno **aggiunti** `parse_rfid` e `parse_ibtn`, oggi assenti.

### E. Frontend

Un solo codebase React, responsive:

- `frontend/src/shell/` — `useBreakpoint()` → `<DesktopShell>` (due pannelli, invariato) vs
  `<MobileShell>` (bottom tab bar: **Radar · File · Libreria · Dispositivo · Impostazioni**).
  Routing con `react-router` (serve il tasto Indietro di Android; l'attuale switch a `useState` in
  `App.tsx:17` non basta).
- `frontend/src/components/radar/`
  - `FlipperSchematic.tsx` — SVG inline del Flipper con hotspot `<g data-chip="subghz">` per ogni chip,
    badge live col conteggio, pulsazione all'arrivo di un nuovo segnale, colore per chip, stato
    grigio/disabilitato per i chip non disponibili nella modalità corrente.
  - `ChipSheet.tsx` — bottom sheet: intestazione "cos'è questo chip" in linguaggio semplice, lista
    segnali live, filtri.
  - `SignalCard.tsx` — tipo, conteggio, primo/ultimo avvistamento, barra RSSI, azioni disponibili.
  - `ActionSheet.tsx` — esecuzione azione con **gate legale**: al primo uso conferma esplicita di
    proprietà/autorizzazione (persistita), poi conferma per ogni trasmissione.
- Stato: il Rust **emette eventi Tauri** (`signal:new`, `signal:update`, `chip:status`) consumati da
  `useSignalStream()`; niente polling lato JS.
- I **10 componenti orfani** già scritti (`SubGhzViewerAdvanced`, `NfcAnalyzerAdvanced`, `IrDatabase`,
  `ReverseEngineerPanel`, `UfbtPanel`, `FapProjectManager`, `FileDiffViewer`, `SerialPanel`,
  `FilePreview`, e i tre viewer base) diventano le viste di dettaglio: vanno **montati**, non
  cancellati — buona parte del "port 1:1" è montare ciò che esiste già. Vanno anche **registrati** i 6
  comandi definiti ma assenti da `lib.rs:38-91` (`parser_parse_*_struct`, `template_*`), che quei
  componenti già invocano e che oggi fallirebbero a runtime.
- `useDirectory.ts` e `useEditor.ts` contengono alberi e contenuti **mock hardcoded**: vanno spostati
  dietro il flag mock esistente e sostituiti dai dati reali.
- i18n: `frontend/src/i18n/{it,en}.ts` + hook `useT()`. Mappa piatta di chiavi con interpolazione,
  ~30 LOC, nessuna libreria.
- **uFBT su mobile:** non può fare shell-out. La vista `UfbtPanel`/`FapProjectManager` su mobile va in
  sola lettura, con messaggio esplicito "disponibile solo su desktop" — non nascosta, così la parità
  1:1 resta onesta.

### F. Build, permessi, CI

- `serialport` va spostato sotto target-cfg, altrimenti il build mobile non parte:
  `[target.'cfg(not(any(target_os="android", target_os="ios")))'.dependencies] serialport = "4"`,
  e `serial`/`transport::usb_cdc` sotto `#[cfg(desktop)]`.
- `run()` in `src-tauri/src/lib.rs:32` va annotato `#[cfg_attr(mobile, tauri::mobile_entry_point)]`;
  togliere `std::process::exit` (`lib.rs:103`), inaccettabile su mobile.
- `tauri android init` / `tauri ios init` → `src-tauri/gen/android`, `src-tauri/gen/apple`; set di
  icone mobile (oggi `src-tauri/icons/` è solo desktop); sezioni `bundle.android`/`bundle.iOS` in
  `src-tauri/tauri.conf.json` (oggi assenti, e la finestra è fissata 1400×900).
- Android `AndroidManifest.xml`: `BLUETOOTH_SCAN`/`BLUETOOTH_CONNECT` (API 31+),
  `ACCESS_FINE_LOCATION` (geo dei rilevamenti + scan BLE su API<31), `POST_NOTIFICATIONS`,
  `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_CONNECTED_DEVICE` (scan in background),
  feature `android.hardware.usb.host` + intent-filter `USB_DEVICE_ATTACHED` con `device_filter.xml`
  (VID 0x0483 / PID 0x5740).
- iOS `Info.plist`: `NSBluetoothAlwaysUsageDescription`, `NSLocationWhenInUseUsageDescription`,
  `UIBackgroundModes: [bluetooth-central]`. Su iOS il BLE in background è fortemente limitato: il
  "background scan" sarà a cadenza ridotta e best-effort, non equivalente ad Android.
- `rusqlite` con feature `bundled` compila SQLite da sorgente: verificare presto che passi con NDK
  Android e toolchain iOS — è un rischio da sondare in Fase 0, non alla fine.
- CI (`.github/workflows/ci.yml`, oggi 6 job ubuntu, zero mobile): aggiungere
  `android-build` (ubuntu + NDK, `tauri android build --apk --debug`) e `ios-check`
  (macos, compile-check su simulatore — sideload, quindi niente firma di distribuzione).

### G. Mock del Flipper

`tools/flipper-mock/` — binario Rust che parla il protobuf RPC del Flipper su TCP, servendo un albero
SD finto (estendere `.flipper_mock/`, oggi 4 file vuoti) e un generatore di eventi di segnale
sintetici per tutti i chip. `transport::mock` si collega lì. Serve a: sviluppo senza hardware, test
end-to-end in CI, e demo dell'onboarding.

---

## Piano per fasi

Ogni fase è spedibile e verificabile da sola; il desktop non deve mai regredire.

| Fase | Contenuto | File principali |
|---|---|---|
| **P0** Fondamenta | cfg-gate `serialport`; `mobile_entry_point`; via `process::exit`; registrare i 6 comandi mancanti; **fix round-trip binario** (aggiungere il decoder base64, oggi c'è solo `base64_encode` in `serial/mod.rs:270` e `serial_upload` rifiuta il non-UTF-8); `tauri android/ios init`; icone mobile; sonda `rusqlite` su NDK | `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/serial/mod.rs`, `src-tauri/tauri.conf.json`, `src-tauri/gen/` |
| **P1** RPC vero | `transport/` + trait; `rpc/session.rs` con connessione persistente e framing; protobuf upstream + prost; `transport/mock.rs` + `tools/flipper-mock/`; desktop USB migrato sul nuovo trait | `src-tauri/src/transport/*`, `src-tauri/src/rpc/*`, `src-tauri/proto/vendor/`, `src-tauri/build.rs` |
| **P2** Radio mobile | plugin BLE (blec o custom Swift/Kotlin); plugin Kotlin USB-OTG; UX di scoperta/pairing/riconnessione | `src-tauri/plugins/flipper-ble/`, `transport/ble.rs`, `transport/usb_otg.rs` |
| **P3** Guscio mobile | `MobileShell` + bottom nav + router; montaggio dei 10 componenti orfani; rimozione dei mock hardcoded dagli hook | `frontend/src/shell/*`, `frontend/src/App.tsx`, `frontend/src/hooks/*` |
| **P4** FAP | sotto-progetto C, protocollo eventi, deploy da app, handshake versione, matrice firmware | `flipper-fap/signal_radar/` |
| **P5** Radar | dominio `signals/` + store SQLite + geo; `FlipperSchematic` + `ChipSheet` + `SignalCard`; stream di eventi; polling adattivo | `src-tauri/src/signals/*`, `frontend/src/components/radar/*` |
| **P6** Azioni | replay SubGHz, emulazione NFC/iButton, invio IR; gate legale; conferme | `src-tauri/src/signals/actions.rs`, `frontend/src/components/radar/ActionSheet.tsx` |
| **P7** Copertura chip | parser `.rfid` e `.ibtn` (assenti); rilevamento moduli esterni; flag di capacità per firmware custom | `src-tauri/src/parsers.rs`, `src-tauri/src/signals/*` |
| **P8** Rifinitura | i18n IT/EN, onboarding, spiegazioni per principianti, scan in background + notifiche, job CI mobile, packaging sideload | `frontend/src/i18n/*`, `.github/workflows/ci.yml` |

---

## Rischi, in ordine di gravità

1. **BLE su Tauri v2 mobile.** Ecosistema immaturo. Mitigazione: sondare in P2 *prima* di costruirci
   sopra; mettere in conto di scrivere il plugin Swift/Kotlin nostro.
2. **Banda BLE ~1–8 KB/s.** Trasferire un `.fap` (decine di KB) o file grandi è lento. Mitigazione:
   deploy del FAP con barra di avanzamento e ripresa; su Android preferire USB-OTG quando presente.
3. **Churn dell'API del FAP tra firmware.** Le API del Flipper cambiano tra versioni e fork.
   Mitigazione: matrice di build + handshake `api_version`, e degradare alla modalità libreria.
4. **Scan BLE ambientale incompatibile con la connessione BLE.** Limite fisico, non aggirabile:
   dichiararlo in UI.
5. **Background scan.** Android richiede un foreground service con notifica persistente; iOS limita
   pesantemente. Aspettarsi comportamenti diversi tra le due piattaforme e dirlo all'utente.
6. **`rusqlite` bundled su NDK/iOS.** Da verificare in P0.
7. **`storage write` del CLI esistente è quasi certamente non funzionante** contro firmware reale
   (il vero comando è multi-linea con terminatore Ctrl-C, non base64 su una riga): non portarlo
   avanti, sostituirlo con `Storage.Write` via RPC.
8. **Ambito enorme.** 8 fasi. Se serve tagliare, P6/P7 sono i candidati a slittare.

---

## Verifica

**Per fase, con il mock:**
```
cargo test && cargo clippy -- -D warnings && cargo fmt --check
cd frontend && npx tsc --noEmit && npx eslint src/ && npm test
cargo run -p flipper-mock &          # emulatore RPC su TCP
npx tauri dev                        # desktop contro il mock
```

**Sul Flipper fisico** (step manuali da chiedere all'utente):
- P1: connessione USB desktop → `System.DeviceInfo` reale, elenco `/ext` reale.
- P2: pairing BLE da Android e da iOS; riconnessione dopo lo spegnimento del Flipper; su Android
  collegare via OTG e verificare che il trasporto passi a USB.
- P4: deploy del `.fap` dall'app, avvio, handshake di versione, ri-deploy dopo bump di versione.
- P5: portare il telefono vicino a un telecomando/cancello → il chip SubGHz pulsa, il conteggio sale,
  il segnale compare con RSSI; avvicinare una carta NFC → il chip NFC reagisce.
- P6: replay di un segnale **proprio**, verificando il gate legale al primo uso.

**Build mobile:**
```
npx tauri android build --apk --debug     # APK sideload
npx tauri ios build --debug               # build dev iOS
```

---

## Esecuzione

L'implementazione va affidata a un agente seguendo
[`docs/AGENT-PROMPT-MOBILE-PORT.md`](./AGENT-PROMPT-MOBILE-PORT.md), che impone il regime
token-efficient concordato: **caveman** (stile di output compresso), **ponytail** (YAGNI prima di ogni
modifica), **RTK** (proxy CLI che tiene fuori dalla chat l'output di build e test) e **Graphify**
(grafo del repo interrogabile invece di leggere file per file).

Nota onesta su Graphify: il repo ha oggi **114 file tracciati**, sotto la soglia dei ~500 dove il
grafo ripaga davvero il costo di costruzione. Va rigenerato quando il port fa crescere il codebase
(realisticamente da P3 in poi), non all'inizio.
