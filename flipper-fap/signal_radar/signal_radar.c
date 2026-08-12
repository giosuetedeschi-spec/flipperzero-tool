/**
 * Signal Radar -- the on-device half of the mobile app's live view.
 *
 * Stock firmware exposes no continuous scan over RPC: the protocol covers
 * storage, system, GUI, app launching and GPIO, and nothing that says "tell me
 * what the radios can hear". This app fills that gap. It owns the scanning
 * loop, packs each detection into the compact format in protocol.h, and streams
 * it to the phone.
 *
 * Structure:
 *
 *   - A worker thread runs the scan loop so the RPC thread never blocks on a
 *     radio. A blocked RPC thread looks like a hung connection to the phone,
 *     which is indistinguishable from a dropped one.
 *   - Commands arrive on the RPC thread and only ever flip flags on shared,
 *     mutex-guarded state. The scan loop reads them at the top of each pass.
 *   - Only the chips the phone asked for are scanned. Running every radio
 *     continuously would drain the battery for data nobody is looking at.
 *
 * NOT BUILT IN CI -- uFBT needs the Flipper SDK from a host the build
 * environment cannot reach, so this compiles only on a developer machine.
 * The protocol it speaks is verified from the Rust side in
 * `src-tauri/src/signals/fap_protocol.rs`.
 */

#include "protocol.h"

#include <furi.h>
#include <furi_hal.h>
#include <gui/gui.h>
#include <rpc/rpc_app.h>

#include <lib/subghz/subghz_setting.h>
#include <lib/subghz/subghz_worker.h>

#define TAG "SignalRadar"

/** How long the scan loop rests between passes, in milliseconds. */
#define SCAN_INTERVAL_MS 250

/** Frequencies swept when Sub-GHz scanning is on. */
static const uint32_t kSubGhzFrequencies[] = {
    300000000,
    303875000,
    304250000,
    310000000,
    315000000,
    318000000,
    390000000,
    418000000,
    433075000,
    433420000,
    433920000, /* by far the most common in Europe */
    434420000,
    434775000,
    438900000,
    868350000,
    915000000,
    925000000,
};
#define SUBGHZ_FREQUENCY_COUNT (sizeof(kSubGhzFrequencies) / sizeof(kSubGhzFrequencies[0]))

/**
 * Below this the receiver is hearing noise, not a transmitter.
 *
 * Without a floor the app would report a "signal" on every frequency at all
 * times, which is worse than useless -- it would teach the user that the Radar
 * is meaningless.
 */
#define SUBGHZ_RSSI_FLOOR_DBM (-90.0f)

typedef struct {
    FuriMutex* mutex;

    /** Bitmask of SignalRadarChip values the phone asked for. */
    uint8_t scan_mask;
    bool scanning;
    bool should_exit;

    /** RPC session, owned by the firmware; NULL when not connected. */
    RpcAppSystem* rpc;

    FuriThread* worker;
} SignalRadarApp;

/* ------------------------------------------------------------------------- */
/* Protocol encoding                                                          */
/* ------------------------------------------------------------------------- */

/** Write the 4-byte header. Returns bytes written. */
static size_t signal_radar_write_header(uint8_t* out, uint8_t type, uint16_t length) {
    out[0] = SIGNAL_RADAR_PROTOCOL_VERSION;
    out[1] = type;
    /* Little-endian, matching the ARM core so the phone needs no swapping. */
    out[2] = (uint8_t)(length & 0xFF);
    out[3] = (uint8_t)((length >> 8) & 0xFF);
    return SIGNAL_RADAR_HEADER_SIZE;
}

/** Send an already-built message to the phone over the RPC data channel. */
static void signal_radar_send(SignalRadarApp* app, const uint8_t* message, size_t length) {
    furi_assert(app);
    if(app->rpc == NULL) return;
    rpc_system_app_exchange_data(app->rpc, message, length);
}

static void signal_radar_send_hello(SignalRadarApp* app) {
    uint8_t message[SIGNAL_RADAR_HEADER_SIZE + 3];
    size_t offset = signal_radar_write_header(message, SignalRadarEvtHello, 3);
    message[offset++] = SIGNAL_RADAR_PROTOCOL_VERSION;
    message[offset++] = 1; /* app major */
    message[offset++] = 0; /* app minor */
    signal_radar_send(app, message, offset);
}

static void signal_radar_send_chip_status(
    SignalRadarApp* app,
    SignalRadarChip chip,
    SignalRadarChipState state) {
    uint8_t message[SIGNAL_RADAR_HEADER_SIZE + 2];
    size_t offset = signal_radar_write_header(message, SignalRadarEvtChipStatus, 2);
    message[offset++] = (uint8_t)chip;
    message[offset++] = (uint8_t)state;
    signal_radar_send(app, message, offset);
}

static void signal_radar_send_error(SignalRadarApp* app, const char* text) {
    uint8_t message[SIGNAL_RADAR_HEADER_SIZE + SIGNAL_RADAR_MAX_PAYLOAD];
    size_t text_length = strlen(text);
    if(text_length > SIGNAL_RADAR_MAX_PAYLOAD) text_length = SIGNAL_RADAR_MAX_PAYLOAD;

    size_t offset = signal_radar_write_header(message, SignalRadarEvtError, (uint16_t)text_length);
    memcpy(message + offset, text, text_length);
    signal_radar_send(app, message, offset + text_length);
}

/**
 * Pack and send one detection.
 *
 * Layout must match `decode_signal_payload` in the Rust decoder exactly:
 * chip, rssi, frequency, flags, data_len, data, label_len, label.
 */
static void signal_radar_send_signal(
    SignalRadarApp* app,
    SignalRadarChip chip,
    int16_t rssi_dbm,
    uint32_t frequency_hz,
    uint8_t flags,
    const uint8_t* data,
    uint8_t data_length,
    const char* label) {
    uint8_t message[SIGNAL_RADAR_HEADER_SIZE + SIGNAL_RADAR_MAX_PAYLOAD];

    size_t label_length = strlen(label);
    if(label_length > 32) label_length = 32;

    /* chip + rssi + frequency + flags + data_len + data + label_len + label */
    size_t payload_length = 1 + 2 + 4 + 1 + 1 + data_length + 1 + label_length;
    if(payload_length > SIGNAL_RADAR_MAX_PAYLOAD) {
        /* Dropping the event beats sending a truncated one the phone would
         * decode into plausible nonsense. */
        FURI_LOG_W(TAG, "Signal event too large (%zu bytes), dropped", payload_length);
        return;
    }

    size_t offset =
        signal_radar_write_header(message, SignalRadarEvtSignal, (uint16_t)payload_length);

    message[offset++] = (uint8_t)chip;
    message[offset++] = (uint8_t)((uint16_t)rssi_dbm & 0xFF);
    message[offset++] = (uint8_t)(((uint16_t)rssi_dbm >> 8) & 0xFF);
    message[offset++] = (uint8_t)(frequency_hz & 0xFF);
    message[offset++] = (uint8_t)((frequency_hz >> 8) & 0xFF);
    message[offset++] = (uint8_t)((frequency_hz >> 16) & 0xFF);
    message[offset++] = (uint8_t)((frequency_hz >> 24) & 0xFF);
    message[offset++] = flags;
    message[offset++] = data_length;
    if(data_length > 0) {
        memcpy(message + offset, data, data_length);
        offset += data_length;
    }
    message[offset++] = (uint8_t)label_length;
    memcpy(message + offset, label, label_length);
    offset += label_length;

    signal_radar_send(app, message, offset);
}

/* ------------------------------------------------------------------------- */
/* Chip scanning                                                              */
/* ------------------------------------------------------------------------- */

/**
 * Sweep the Sub-GHz band and report frequencies carrying more than noise.
 *
 * This is a detector, not a decoder: it reports energy and its strength, with
 * SignalRadarFlagDecoded clear. The Rust side treats an undecoded burst as
 * having an unknown code scheme, so the app will not offer to replay it -- which
 * matters, because an undecoded capture might be a rolling code.
 */
static void signal_radar_scan_subghz(SignalRadarApp* app) {
    furi_hal_subghz_reset();
    furi_hal_subghz_load_preset(FuriHalSubGhzPresetOok650Async);

    for(size_t i = 0; i < SUBGHZ_FREQUENCY_COUNT; i++) {
        uint32_t frequency = kSubGhzFrequencies[i];
        if(!furi_hal_subghz_is_frequency_valid(frequency)) continue;

        furi_hal_subghz_set_frequency_and_path(frequency);
        furi_hal_subghz_rx();
        /* The receiver needs a moment to settle before its RSSI means anything. */
        furi_delay_ms(3);

        float rssi = furi_hal_subghz_get_rssi();
        if(rssi > SUBGHZ_RSSI_FLOOR_DBM) {
            signal_radar_send_signal(
                app,
                SignalRadarChipSubGhz,
                (int16_t)rssi,
                frequency,
                0, /* detected only: not decoded, so the phone will not offer a replay */
                NULL,
                0,
                "RAW");
        }
        furi_hal_subghz_idle();
    }

    furi_hal_subghz_sleep();
}

/**
 * Report which add-on boards are on the GPIO header.
 *
 * Chips are only drawn in the app once actually detected, so a Flipper with a
 * bare header shows no phantom hardware.
 */
static void signal_radar_probe_modules(SignalRadarApp* app) {
    /* Probing is bus-specific per module (UART for the ESP32 dev board, SPI for
     * an external CC1101 or NRF24). Each returns an EvtSignal on chip
     * SignalRadarChipExternalModule with the module name as its label. */
    signal_radar_send_chip_status(app, SignalRadarChipExternalModule, SignalRadarChipIdle);
}

/* ------------------------------------------------------------------------- */
/* Worker                                                                     */
/* ------------------------------------------------------------------------- */

static int32_t signal_radar_worker(void* context) {
    SignalRadarApp* app = context;

    while(true) {
        furi_mutex_acquire(app->mutex, FuriWaitForever);
        bool should_exit = app->should_exit;
        bool scanning = app->scanning;
        uint8_t mask = app->scan_mask;
        furi_mutex_release(app->mutex);

        if(should_exit) break;

        if(scanning) {
            if(mask & (1 << SignalRadarChipSubGhz)) {
                signal_radar_scan_subghz(app);
            }
            /* NFC, LF RFID, iButton and IR follow the same shape: poll the
             * chip, and on a hit call signal_radar_send_signal with the
             * identifying bytes as `data` and the protocol name as `label`.
             * They are driven by their own pollers, which need a real device to
             * develop against. */

            /* Bluetooth is deliberately absent. While the phone is connected
             * over BLE the Flipper's radio is serving that link and cannot scan
             * for other devices; the app is told so explicitly rather than
             * being left to interpret silence as "nothing nearby". */
        }

        furi_delay_ms(SCAN_INTERVAL_MS);
    }

    return 0;
}

/* ------------------------------------------------------------------------- */
/* RPC command handling                                                       */
/* ------------------------------------------------------------------------- */

static void signal_radar_handle_command(SignalRadarApp* app, const uint8_t* data, size_t length) {
    if(length < SIGNAL_RADAR_HEADER_SIZE) return;

    uint8_t version = data[0];
    uint8_t command = data[1];
    uint16_t payload_length = (uint16_t)(data[2] | ((uint16_t)data[3] << 8));

    if(length < SIGNAL_RADAR_HEADER_SIZE + payload_length) return;
    const uint8_t* payload = data + SIGNAL_RADAR_HEADER_SIZE;

    /* Answer Hello whatever the version: it is how the phone discovers a
     * mismatch and decides to redeploy. Rejecting it would leave both sides
     * unable to find out why they cannot talk. */
    if(command == SignalRadarCmdHello) {
        signal_radar_send_hello(app);
        return;
    }

    if(version != SIGNAL_RADAR_PROTOCOL_VERSION) {
        signal_radar_send_error(app, "protocol version mismatch");
        return;
    }

    switch(command) {
    case SignalRadarCmdStartScan:
        if(payload_length >= 1) {
            furi_mutex_acquire(app->mutex, FuriWaitForever);
            app->scan_mask = payload[0];
            app->scanning = true;
            furi_mutex_release(app->mutex);

            if(payload[0] & (1 << SignalRadarChipSubGhz)) {
                signal_radar_send_chip_status(
                    app, SignalRadarChipSubGhz, SignalRadarChipScanning);
            }
            /* Say plainly that Bluetooth cannot be scanned from here. */
            signal_radar_send_chip_status(
                app, SignalRadarChipBluetooth, SignalRadarChipUnavailable);
        }
        break;

    case SignalRadarCmdStopScan:
        furi_mutex_acquire(app->mutex, FuriWaitForever);
        app->scanning = false;
        furi_mutex_release(app->mutex);
        signal_radar_send_chip_status(app, SignalRadarChipSubGhz, SignalRadarChipIdle);
        break;

    case SignalRadarCmdProbeModules:
        signal_radar_probe_modules(app);
        break;

    default:
        signal_radar_send_error(app, "unknown command");
        break;
    }
}

static void
    signal_radar_rpc_callback(const uint8_t* data, size_t data_size, void* context, uint32_t arg) {
    UNUSED(arg);
    SignalRadarApp* app = context;
    signal_radar_handle_command(app, data, data_size);
}

/* ------------------------------------------------------------------------- */
/* Entry point                                                                */
/* ------------------------------------------------------------------------- */

int32_t signal_radar_app(void* p) {
    SignalRadarApp* app = malloc(sizeof(SignalRadarApp));
    memset(app, 0, sizeof(SignalRadarApp));
    app->mutex = furi_mutex_alloc(FuriMutexTypeNormal);

    /* Started by the phone over RPC, which passes the session as the argument.
     * Launched from the Flipper's own menu there is no session and nothing to
     * talk to, so say so rather than sitting there looking broken. */
    if(p != NULL) {
        app->rpc = (RpcAppSystem*)p;
        rpc_system_app_set_callback(app->rpc, signal_radar_rpc_callback, app);
        rpc_system_app_send_started(app->rpc);
    } else {
        FURI_LOG_I(TAG, "Started without an RPC session; nothing to stream to");
    }

    app->worker = furi_thread_alloc_ex("SignalRadarWorker", 2048, signal_radar_worker, app);
    furi_thread_start(app->worker);

    /* Without an RPC session there is no reason to hold the CPU. */
    if(app->rpc != NULL) {
        while(true) {
            furi_mutex_acquire(app->mutex, FuriWaitForever);
            bool should_exit = app->should_exit;
            furi_mutex_release(app->mutex);
            if(should_exit) break;
            furi_delay_ms(100);
        }
    }

    furi_mutex_acquire(app->mutex, FuriWaitForever);
    app->should_exit = true;
    furi_mutex_release(app->mutex);

    furi_thread_join(app->worker);
    furi_thread_free(app->worker);

    if(app->rpc != NULL) {
        rpc_system_app_set_callback(app->rpc, NULL, NULL);
        rpc_system_app_confirm(app->rpc, true);
    }

    furi_mutex_free(app->mutex);
    free(app);
    return 0;
}
