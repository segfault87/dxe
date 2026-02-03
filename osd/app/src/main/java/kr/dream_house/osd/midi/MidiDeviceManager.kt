package kr.dream_house.osd.midi

import android.content.Context
import android.content.pm.PackageManager
import android.media.midi.MidiDevice
import android.media.midi.MidiDeviceInfo
import android.media.midi.MidiInputPort
import android.media.midi.MidiManager
import android.media.midi.MidiReceiver
import android.os.Build
import android.os.Handler
import android.util.Log
import java.io.IOException
import java.util.concurrent.Executors

fun Context.createMidiDeviceManager(): MidiDeviceManager? {
    return if (packageManager.hasSystemFeature(PackageManager.FEATURE_MIDI)) {
        MidiDeviceManager(getSystemService(MidiManager::class.java))
    } else {
        null
    }
}

interface MidiDeviceEventHandler {
    fun onReceive(payload: ByteArray, offset: Int, count: Int)
    fun onConnect()
    fun onDisconnect()
}

class MidiDeviceManager(private val midiManager: MidiManager) : MidiManager.DeviceCallback(), MidiDeviceEventHandler {
    companion object {
        private const val TAG = "MidiDeviceManager"

        private const val DESIRED_PORT_INPUT = 0
        private const val DESIRED_PORT_OUTPUT = 0

        private val SYSEX_IDENTITY_REQUEST = byteArrayOf(0xf0.toByte(), 0x7e, 0x7f, 0x06, 0x01, 0xf7.toByte())
    }

    private var currentDevice: MidiDevice? = null
    private var currentDeviceSerial: String? = null
    private var inputPorts = mutableListOf<MidiInputPort>()

    private val executor = Executors.newFixedThreadPool(1)
    private val handler = Handler()

    private var handlers = mutableListOf<MidiDeviceEventHandler>()

    init {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            midiManager.registerDeviceCallback(MidiManager.TRANSPORT_MIDI_BYTE_STREAM, executor, this)
        } else {
            midiManager.registerDeviceCallback(this, handler)
        }

        val infos = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            midiManager.getDevicesForTransport(MidiManager.TRANSPORT_MIDI_BYTE_STREAM).toList()
        } else {
            midiManager.devices.toList()
        }

        addHandler(this)

        // Pick first device
        infos.firstOrNull()?.let {
            initializeDevice(it)
        }
    }

    private fun initializeDevice(deviceInfo: MidiDeviceInfo) {
        midiManager.openDevice(deviceInfo, object: MidiManager.OnDeviceOpenedListener {
            override fun onDeviceOpened(device: MidiDevice?) {
                if (device == null) {
                    Log.e(TAG, "Couldn't open MIDI device.")
                    return
                }

                Log.i(TAG, "Opened MIDI device: $device")

                val outputPortNumber = if (deviceInfo.outputPortCount - 1 > DESIRED_PORT_OUTPUT) {
                    Log.w(TAG, "MIDI device $deviceInfo has fewer output port count than desired value. defaulting to 0...")
                    0
                } else {
                    DESIRED_PORT_OUTPUT
                }

                for (i in 0 until deviceInfo.inputPortCount) {
                    inputPorts.add(device.openInputPort(i))
                }

                val outputPort = device.openOutputPort(outputPortNumber)
                Log.d(TAG, "Output output port $outputPortNumber complete")
                outputPort.connect(object: MidiReceiver() {
                    override fun onSend(
                        msg: ByteArray,
                        offset: Int,
                        count: Int,
                        timestamp: Long
                    ) {
                        for (handler in handlers) {
                            handler.onReceive(msg, offset, count)
                        }
                    }
                })

                for (handler in handlers) {
                    handler.onConnect()
                }

                currentDeviceSerial = deviceInfo.properties.getString("serial_number")
                currentDevice = device
            }
        }, null)
    }

    fun addHandler(handler: MidiDeviceEventHandler) {
        handlers.add(handler)
        if (currentDevice != null) {
            handler.onConnect()
        } else {
            handler.onDisconnect()
        }
    }

    fun removeHandler(handler: MidiDeviceEventHandler) {
        handlers.remove(handler)
    }

    private fun sendBulkPayloads(payloads: List<ByteArray>, timeInterval: Long, port: Int = 0) {
        val port = inputPorts.getOrNull(port)
        if (port == null) {
            Log.e(TAG, "Trying to send while input port is not opened.")
            return
        }

        postEvent {
            try {
                for (payload in payloads) {
                    port.send(payload, 0, payload.size)
                    Thread.sleep(timeInterval)
                }
            } catch (e: IOException) {
                Log.e(TAG, "Can't send MIDI message to port: $e")
            }
        }
    }

    fun enqueueBulkPayloads(payloads: List<ByteArray>, timeInterval: Long) {
        postEvent {
            sendBulkPayloads(payloads, timeInterval)
        }
    }

    private var identityRequestCallbacks = mutableListOf<() -> Unit>()

    fun addIdentityRequestCallback(fn: () -> Unit) {
        identityRequestCallbacks.add(fn)
    }

    fun removeIdentityRequestCallback(fn: () -> Unit) {
        identityRequestCallbacks.remove(fn)
    }

    fun sendIdentityRequest() {
        send(SYSEX_IDENTITY_REQUEST, 0, SYSEX_IDENTITY_REQUEST.size)
    }

    fun send(payload: ByteArray, offset: Int, count: Int, port: Int = 0, timestamp: Long? = null): Boolean {
        val port = inputPorts.getOrNull(port)
        return if (port == null) {
            Log.e(TAG, "Trying to send while input port is not opened.")
            false
        } else {
            postEvent {
                try {
                    if (timestamp != null) {
                        port.send(payload, offset, count, timestamp)
                    } else {
                        port.send(payload, offset, count)
                    }
                } catch (e: IOException) {
                    Log.e(TAG, "Can't send MIDI message to port: $e")
                }
            }

            true
        }
    }

    private fun postEvent(task: () -> Unit) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            executor.submit(task)
        } else {
            handler.post(task)
        }
    }

    override fun onDeviceAdded(device: MidiDeviceInfo) {
        super.onDeviceAdded(device)

        Log.i(TAG, "onDeviceAdded: $device")

        if (currentDevice == null) {
            initializeDevice(device)
        }
    }

    override fun onDeviceRemoved(device: MidiDeviceInfo) {
        super.onDeviceRemoved(device)

        val removedDeviceSerial = device.properties.getString("serial_number")
        if (currentDeviceSerial == removedDeviceSerial) {
            for (handler in handlers) {
                handler.onDisconnect()
            }
        }

        currentDevice?.close()
        currentDevice = null
        inputPorts.clear();
    }

    override fun onReceive(payload: ByteArray, offset: Int, count: Int) {
        if (count > 0 && payload[offset] == 0xf0.toByte()) {
            for (fn in identityRequestCallbacks) {
                fn()
            }
        }
    }

    override fun onConnect() {}

    override fun onDisconnect() {}
}