package com.giosue.flipperzero.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothProfile
import android.content.Context
import java.util.UUID
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * Android half of the BLE link to a Flipper Zero.
 *
 * Mirrors the `BleLink` trait in `src-tauri/src/transport/ble.rs`: it moves
 * bytes and reports two numbers, and knows nothing about RPC framing. All the
 * protocol logic -- chunking, flow control waiting, frame reassembly -- lives on
 * the Rust side, where it is unit tested. This class only has to be a correct
 * GATT client.
 *
 * NOT VERIFIED IN CI. There is no Android device or emulator in the build, so
 * this is exercised only against real hardware.
 */
class FlipperBleLink(private val context: Context) {

    companion object {
        val SERIAL_SERVICE: UUID = UUID.fromString("8fe5b3d5-2e7f-4a98-2a48-7acc60fe0000")
        val RX_CHARACTERISTIC: UUID = UUID.fromString("19ed82ae-ed21-4c9d-4145-228e61fe0001")
        val TX_CHARACTERISTIC: UUID = UUID.fromString("19ed82ae-ed21-4c9d-4145-228e61fe0002")
        val FLOW_CONTROL_CHARACTERISTIC: UUID =
            UUID.fromString("19ed82ae-ed21-4c9d-4145-228e61fe0003")

        /** Standard descriptor every notification subscription must write. */
        val CLIENT_CHARACTERISTIC_CONFIG: UUID =
            UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")

        /**
         * Largest MTU worth asking for. Android caps at 517; the Flipper will
         * negotiate down to whatever it supports, so asking high costs nothing.
         */
        private const val DESIRED_MTU = 512

        /** ATT protocol overhead subtracted from the MTU to get usable payload. */
        private const val ATT_HEADER_BYTES = 3
    }

    private var gatt: BluetoothGatt? = null

    /**
     * Notification bytes waiting to be drained by Rust.
     *
     * A queue rather than a buffer because notifications arrive on the GATT
     * callback thread while Rust drains from another; the concurrent queue is
     * what keeps that safe without a lock on the callback path.
     */
    private val inbox = ConcurrentLinkedQueue<Byte>()

    private val connected = AtomicBoolean(false)
    private val mtuPayload = AtomicInteger(20)
    /** -1 means the device has not published flow control. */
    private val freeBuffer = AtomicInteger(-1)

    private val callback = object : BluetoothGattCallback() {

        override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    connected.set(true)
                    // Raise the MTU before discovering services: a 20-byte
                    // payload makes every transfer roughly ten times slower
                    // than it needs to be.
                    gatt.requestMtu(DESIRED_MTU)
                }
                BluetoothProfile.STATE_DISCONNECTED -> {
                    connected.set(false)
                    mtuPayload.set(20)
                    freeBuffer.set(-1)
                }
            }
        }

        override fun onMtuChanged(gatt: BluetoothGatt, mtu: Int, status: Int) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                mtuPayload.set((mtu - ATT_HEADER_BYTES).coerceAtLeast(20))
            }
            // Discover only once the MTU is settled, so the characteristics are
            // used at their final size from the first write.
            gatt.discoverServices()
        }

        override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
            if (status != BluetoothGatt.GATT_SUCCESS) return
            val service = gatt.getService(SERIAL_SERVICE) ?: return

            service.getCharacteristic(TX_CHARACTERISTIC)?.let { subscribe(gatt, it) }
            // Flow control is optional on older firmware; absence simply means
            // the Rust side will not throttle.
            service.getCharacteristic(FLOW_CONTROL_CHARACTERISTIC)?.let { subscribe(gatt, it) }
        }

        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            value: ByteArray,
        ) {
            when (characteristic.uuid) {
                TX_CHARACTERISTIC -> value.forEach { inbox.add(it) }
                FLOW_CONTROL_CHARACTERISTIC -> freeBuffer.set(decodeFreeBuffer(value))
            }
        }

        @Deprecated("Pre-API 33 callback; kept so older devices still deliver data")
        @Suppress("DEPRECATION")
        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
        ) {
            val value = characteristic.value ?: return
            when (characteristic.uuid) {
                TX_CHARACTERISTIC -> value.forEach { inbox.add(it) }
                FLOW_CONTROL_CHARACTERISTIC -> freeBuffer.set(decodeFreeBuffer(value))
            }
        }
    }

    /** The Flipper publishes free buffer space as a little-endian value. */
    private fun decodeFreeBuffer(value: ByteArray): Int {
        var result = 0
        for (i in value.indices.reversed()) {
            result = (result shl 8) or (value[i].toInt() and 0xFF)
        }
        return result
    }

    @SuppressLint("MissingPermission")
    private fun subscribe(gatt: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
        gatt.setCharacteristicNotification(characteristic, true)
        // Enabling notifications locally is not enough: without writing the
        // CCC descriptor the peripheral never actually sends anything, and the
        // link looks connected but permanently silent.
        characteristic.getDescriptor(CLIENT_CHARACTERISTIC_CONFIG)?.let { descriptor ->
            @Suppress("DEPRECATION")
            descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
            @Suppress("DEPRECATION")
            gatt.writeDescriptor(descriptor)
        }
    }

    @SuppressLint("MissingPermission")
    fun connect(device: BluetoothDevice) {
        gatt = device.connectGatt(context, false, callback, BluetoothDevice.TRANSPORT_LE)
    }

    @SuppressLint("MissingPermission")
    fun disconnect() {
        gatt?.disconnect()
        gatt?.close()
        gatt = null
        connected.set(false)
        inbox.clear()
    }

    /** Write one chunk. The caller has already sized it to [mtuPayload]. */
    @SuppressLint("MissingPermission")
    fun writeChunk(chunk: ByteArray): Boolean {
        val activeGatt = gatt ?: return false
        val characteristic = activeGatt
            .getService(SERIAL_SERVICE)
            ?.getCharacteristic(RX_CHARACTERISTIC)
            ?: return false

        // Write-without-response is what makes throughput usable; the Flipper's
        // own flow-control characteristic replaces the per-write acknowledgement
        // that would otherwise pace us.
        return if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            activeGatt.writeCharacteristic(
                characteristic,
                chunk,
                BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE,
            ) == BluetoothGatt.GATT_SUCCESS
        } else {
            @Suppress("DEPRECATION")
            characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
            @Suppress("DEPRECATION")
            characteristic.value = chunk
            @Suppress("DEPRECATION")
            activeGatt.writeCharacteristic(characteristic)
        }
    }

    /** Hand over everything received since the last call. */
    fun drainNotifications(): ByteArray {
        val out = ArrayList<Byte>(inbox.size)
        while (true) {
            val byte = inbox.poll() ?: break
            out.add(byte)
        }
        return out.toByteArray()
    }

    fun mtuPayload(): Int = mtuPayload.get()

    /** Free device buffer, or -1 when the device does not report it. */
    fun freeDeviceBuffer(): Int = freeBuffer.get()

    fun isConnected(): Boolean = connected.get()
}
