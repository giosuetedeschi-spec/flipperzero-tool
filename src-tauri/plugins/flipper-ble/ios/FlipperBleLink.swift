import CoreBluetooth
import Foundation

/// iOS half of the BLE link to a Flipper Zero.
///
/// Mirrors the `BleLink` trait in `src-tauri/src/transport/ble.rs`: it moves
/// bytes and reports two numbers, and knows nothing about RPC framing. All the
/// protocol logic -- chunking, flow-control waiting, frame reassembly -- lives on
/// the Rust side, where it is unit tested. This type only has to be a correct
/// Central.
///
/// On iOS this is the *only* way to reach the device: USB serial requires MFi,
/// so there is no fallback link on this platform.
///
/// NOT VERIFIED IN CI. Building for iOS needs macOS and Xcode, neither of which
/// exists in the build environment, so this is exercised only on real hardware.
final class FlipperBleLink: NSObject {

    private enum GATT {
        static let serialService = CBUUID(string: "8FE5B3D5-2E7F-4A98-2A48-7ACC60FE0000")
        static let rxCharacteristic = CBUUID(string: "19ED82AE-ED21-4C9D-4145-228E61FE0001")
        static let txCharacteristic = CBUUID(string: "19ED82AE-ED21-4C9D-4145-228E61FE0002")
        static let flowControlCharacteristic = CBUUID(string: "19ED82AE-ED21-4C9D-4145-228E61FE0003")
    }

    /// Conservative payload until the peripheral reports its real limit.
    private static let defaultMTUPayload = 20

    private var central: CBCentralManager!
    private var peripheral: CBPeripheral?
    private var rxCharacteristic: CBCharacteristic?

    /// Received bytes waiting for Rust to drain them.
    ///
    /// Guarded by a lock because CoreBluetooth delivers on its own queue while
    /// Rust drains from another thread.
    private var inbox = [UInt8]()
    private let inboxLock = NSLock()

    private var connected = false
    /// -1 means the device has not published flow control.
    private var freeBuffer: Int = -1

    override init() {
        super.init()
        central = CBCentralManager(delegate: self, queue: nil)
    }

    // MARK: - Bridge surface

    func startScan() {
        // Filtering by service UUID is required, not just polite: iOS will not
        // return peripherals in the background otherwise.
        central.scanForPeripherals(withServices: [GATT.serialService], options: nil)
    }

    func connect(to peripheral: CBPeripheral) {
        self.peripheral = peripheral
        peripheral.delegate = self
        central.stopScan()
        central.connect(peripheral, options: nil)
    }

    func disconnect() {
        if let peripheral = peripheral {
            central.cancelPeripheralConnection(peripheral)
        }
        peripheral = nil
        rxCharacteristic = nil
        connected = false
        inboxLock.lock()
        inbox.removeAll()
        inboxLock.unlock()
    }

    /// Write one chunk. The caller has already sized it to `mtuPayload()`.
    @discardableResult
    func writeChunk(_ chunk: Data) -> Bool {
        guard let peripheral = peripheral, let characteristic = rxCharacteristic else {
            return false
        }
        // Without response is what makes throughput usable; the Flipper's own
        // flow-control characteristic replaces the per-write acknowledgement
        // that would otherwise pace us.
        peripheral.writeValue(chunk, for: characteristic, type: .withoutResponse)
        return true
    }

    /// Hand over everything received since the last call.
    func drainNotifications() -> Data {
        inboxLock.lock()
        defer { inboxLock.unlock() }
        let data = Data(inbox)
        inbox.removeAll(keepingCapacity: true)
        return data
    }

    func mtuPayload() -> Int {
        guard let peripheral = peripheral else { return Self.defaultMTUPayload }
        // iOS reports the usable payload directly, with ATT overhead already
        // removed -- unlike Android, where it must be subtracted by hand.
        return max(peripheral.maximumWriteValueLength(for: .withoutResponse), Self.defaultMTUPayload)
    }

    /// Free device buffer, or -1 when the device does not report it.
    func freeDeviceBuffer() -> Int { freeBuffer }

    func isConnected() -> Bool { connected }

    // MARK: - Helpers

    /// The Flipper publishes free buffer space little-endian.
    private func decodeFreeBuffer(_ data: Data) -> Int {
        var result = 0
        for byte in data.reversed() {
            result = (result << 8) | Int(byte)
        }
        return result
    }
}

// MARK: - CBCentralManagerDelegate

extension FlipperBleLink: CBCentralManagerDelegate {

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        if central.state == .poweredOn {
            startScan()
        } else {
            connected = false
        }
    }

    func centralManager(
        _ central: CBCentralManager,
        didConnect peripheral: CBPeripheral
    ) {
        connected = true
        peripheral.discoverServices([GATT.serialService])
    }

    func centralManager(
        _ central: CBCentralManager,
        didDisconnectPeripheral peripheral: CBPeripheral,
        error: Error?
    ) {
        connected = false
        rxCharacteristic = nil
        freeBuffer = -1
    }
}

// MARK: - CBPeripheralDelegate

extension FlipperBleLink: CBPeripheralDelegate {

    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        guard error == nil, let services = peripheral.services else { return }
        for service in services where service.uuid == GATT.serialService {
            peripheral.discoverCharacteristics(
                [GATT.rxCharacteristic, GATT.txCharacteristic, GATT.flowControlCharacteristic],
                for: service
            )
        }
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didDiscoverCharacteristicsFor service: CBService,
        error: Error?
    ) {
        guard error == nil, let characteristics = service.characteristics else { return }

        for characteristic in characteristics {
            switch characteristic.uuid {
            case GATT.rxCharacteristic:
                rxCharacteristic = characteristic
            case GATT.txCharacteristic:
                peripheral.setNotifyValue(true, for: characteristic)
            case GATT.flowControlCharacteristic:
                // Optional on older firmware; absence just means Rust will not
                // throttle writes.
                peripheral.setNotifyValue(true, for: characteristic)
            default:
                break
            }
        }
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didUpdateValueFor characteristic: CBCharacteristic,
        error: Error?
    ) {
        guard error == nil, let value = characteristic.value else { return }

        switch characteristic.uuid {
        case GATT.txCharacteristic:
            inboxLock.lock()
            inbox.append(contentsOf: value)
            inboxLock.unlock()
        case GATT.flowControlCharacteristic:
            freeBuffer = decodeFreeBuffer(value)
        default:
            break
        }
    }
}
