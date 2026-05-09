package kr.dream_house.osd.midi

import android.content.Context
import android.content.pm.PackageManager
import android.media.midi.MidiDevice
import android.media.midi.MidiDeviceInfo
import android.media.midi.MidiInputPort
import android.media.midi.MidiManager
import android.media.midi.MidiOutputPort
import android.media.midi.MidiReceiver
import android.os.Build
import android.os.Handler
import android.util.Log
import kr.dream_house.osd.midi.devices.AhSqDevice
import kr.dream_house.osd.midi.devices.Vm3100ProMixerDevice
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
    fun onConnect(mixer: MixerDevice)
    fun onDisconnect()
}

private fun parseSysExIdentityResponse(data: ByteArray, offset: Int): MidiDeviceIdentifier? {
    if (data.size - offset < 6) {
        return null
    }

    if (data[offset] != 0xf0.toByte() || data[offset + 3] != 0x06.toByte() || data[offset + 4] != 0x02.toByte()) {
        return null
    }

    var end = 0
    for (i in 5 until data.size) {
        if (data[i] == 0xf7.toByte()) {
            end = i
            break
        }
    }

    val manufacturerId: ByteArray
    val deviceFamily: ByteArray
    val familyMember: ByteArray
    val revision: ByteArray
    if (end - offset == 14) {
        // 1 byte manufacturer
        manufacturerId = byteArrayOf(data[offset + 5])
        deviceFamily = data.copyOfRange(offset + 6, offset + 8)
        familyMember = data.copyOfRange(offset + 8, offset + 10)
        revision = data.copyOfRange(offset + 10, offset + 14)
    } else if (end - offset == 16) {
        // 3 bytes manufacturer
        manufacturerId = data.copyOfRange(offset + 5, offset + 8)
        deviceFamily = data.copyOfRange(offset + 8, offset + 10)
        familyMember = data.copyOfRange(offset + 10, offset + 12)
        revision = data.copyOfRange(offset + 12, offset + 16)
    } else {
        return null
    }

    return MidiDeviceIdentifier(
        manufacturerId = manufacturerId,
        deviceFamily = deviceFamily,
        familyMember = familyMember,
        revision = revision,
    )
}

class MidiDeviceManager(private val midiManager: MidiManager) : MidiManager.DeviceCallback(), MidiDeviceEventHandler {
    companion object {
        private const val TAG = "MidiDeviceManager"

        private val SYSEX_IDENTITY_REQUEST = byteArrayOf(0xf0.toByte(), 0x7e, 0x7f, 0x06, 0x01, 0xf7.toByte())
    }

    private val specs: List<MidiMixerSpec> = listOf(
        Vm3100ProMixerDevice,
        AhSqDevice,
    )

    var currentMixer: MixerDevice? = null
        private set
    private var currentDevice: MidiDevice? = null
    private var inputPort: MidiInputPort? = null
    private var outputPort: MidiOutputPort? = null
    private var currentDeviceSerial: String? = null

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

        for (deviceInfo in infos) {
            probeDevice(deviceInfo)
        }
    }

    private fun probeDevice(deviceInfo: MidiDeviceInfo) {
        val inputPorts = mutableListOf<MidiInputPort>()
        val outputPorts = mutableListOf<MidiOutputPort>()
        var openedDevice: MidiDevice? = null

        var inputIndex = -1

        var desiredDevice: MixerDevice? = null
        for (spec in specs) {
            desiredDevice = spec.probe(deviceInfo.properties)
            if (desiredDevice != null) {
                break
            }
        }

        if (desiredDevice == null) {
            val timeoutTask = object: Runnable {
                override fun run() {
                    if (desiredDevice != null) {
                        return
                    }

                    if (inputIndex + 1 >= inputPorts.size) {
                        Log.d(TAG, "Failed to probe")
                        for (port in inputPorts) {
                            port.close()
                        }
                        for (port in outputPorts) {
                            port.close()
                        }
                        openedDevice?.close()
                    } else {
                        inputIndex += 1
                        Log.d(TAG, "Sending identity request to $inputIndex")
                        inputPorts[inputIndex].send(SYSEX_IDENTITY_REQUEST, 0, SYSEX_IDENTITY_REQUEST.size)

                        handler.postDelayed(this, 500)
                    }
                }
            }
            handler.postDelayed(timeoutTask, 500)
        }

        midiManager.openDevice(deviceInfo, object: MidiManager.OnDeviceOpenedListener {
            override fun onDeviceOpened(device: MidiDevice?) {
                if (device == null) {
                    Log.e(TAG, "Couldn't open MIDI device.")
                    return
                }

                if (desiredDevice != null) {
                    this@MidiDeviceManager.inputPort = device.openInputPort(0)
                    val outputPort = device.openOutputPort(0)
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
                    this@MidiDeviceManager.outputPort = outputPort
                    currentDevice = device
                    currentMixer = desiredDevice
                    currentDeviceSerial = deviceInfo.properties.getString("serial_number")
                    for (handler in handlers) {
                        Log.d(TAG, "Mixer connect $desiredDevice")
                        handler.onConnect(desiredDevice!!)
                    }
                } else {
                    openedDevice = device

                    for (i in 0 until deviceInfo.inputPortCount) {
                        val inputPort = device.openInputPort(i)
                        inputPorts.add(inputPort)
                    }

                    for (i in 0 until deviceInfo.outputPortCount) {
                        val outputPort = device.openOutputPort(i)
                        outputPort.connect(object : MidiReceiver() {
                            override fun onSend(
                                msg: ByteArray,
                                offset: Int,
                                count: Int,
                                timestamp: Long
                            ) {
                                if (this@MidiDeviceManager.outputPort == null) {
                                    // Probe device
                                    if (msg[offset] == 0xf0.toByte()) {
                                        val deviceIdentifier =
                                            parseSysExIdentityResponse(msg, offset)
                                        if (deviceIdentifier != null) {
                                            this@MidiDeviceManager.inputPort =
                                                inputPorts[inputIndex]
                                            this@MidiDeviceManager.outputPort = outputPort
                                            for (spec in specs) {
                                                desiredDevice = spec.probe(deviceIdentifier)
                                                if (desiredDevice != null) {
                                                    break
                                                }
                                            }

                                            if (desiredDevice != null) {
                                                currentMixer = desiredDevice
                                                currentDevice = device
                                                currentDeviceSerial =
                                                    deviceInfo.properties.getString("serial_number")
                                                for (handler in handlers) {
                                                    handler.onConnect(desiredDevice!!)
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    for (handler in handlers) {
                                        handler.onReceive(msg, offset, count)
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }, null)
    }

    fun addHandler(handler: MidiDeviceEventHandler) {
        handlers.add(handler)
        if (currentMixer != null) {
            handler.onConnect(currentMixer!!)
        } else {
            handler.onDisconnect()
        }
    }

    fun removeHandler(handler: MidiDeviceEventHandler) {
        handlers.remove(handler)
    }

    private fun sendBulkPayloads(payloads: List<ByteArray>, timeInterval: Long) {
        if (inputPort == null) {
            Log.e(TAG, "Trying to send while input port is not opened.")
            return
        }

        val inputPort = inputPort!!

        postEvent {
            try {
                for (payload in payloads) {
                    inputPort.send(payload, 0, payload.size)
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

    fun send(payload: ByteArray, offset: Int, count: Int, timestamp: Long? = null): Boolean {
        return if (inputPort == null) {
            Log.e(TAG, "Trying to send while input port is not opened.")
            false
        } else {
            val inputPort = inputPort!!
            postEvent {
                try {
                    if (timestamp != null) {
                        inputPort.send(payload, offset, count, timestamp)
                    } else {
                        inputPort.send(payload, offset, count)
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

        if (currentMixer == null) {
            probeDevice(device)
        }
    }

    override fun onDeviceRemoved(device: MidiDeviceInfo) {
        super.onDeviceRemoved(device)

        val removedDeviceSerial = device.properties.getString("serial_number")
        if (currentDeviceSerial != removedDeviceSerial) {
            return
        }

        inputPort?.close()
        inputPort = null
        outputPort?.close()
        outputPort = null
        currentDevice?.close()
        currentDevice = null
        currentMixer = null

        for (handler in handlers) {
            handler.onDisconnect()
        }
    }

    override fun onReceive(payload: ByteArray, offset: Int, count: Int) {
        if (count > 0 && payload[offset] == 0xf0.toByte()) {
            for (fn in identityRequestCallbacks) {
                fn()
            }
        }
    }

    override fun onConnect(mixer: MixerDevice) {}

    override fun onDisconnect() {}
}