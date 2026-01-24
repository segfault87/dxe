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

class MidiDeviceManager(private val midiManager: MidiManager) : MidiManager.DeviceCallback() {
    companion object {
        private const val TAG = "MidiDeviceManager"

        private const val DESIRED_PORT_INPUT = 0
        private const val DESIRED_PORT_OUTPUT = 0
    }

    private var currentDevice: MidiDevice? = null
    private var inputPort: MidiInputPort? = null

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

                val inputPortNumber = if (deviceInfo.inputPortCount - 1 > DESIRED_PORT_INPUT) {
                    Log.w(TAG, "MIDI device $deviceInfo has fewer input port count than desired value. defaulting to 0...")
                    0
                } else {
                    DESIRED_PORT_INPUT
                }

                val outputPortNumber = if (deviceInfo.outputPortCount - 1 > DESIRED_PORT_OUTPUT) {
                    Log.w(TAG, "MIDI device $deviceInfo has fewer output port count than desired value. defaulting to 0...")
                    0
                } else {
                    DESIRED_PORT_OUTPUT
                }

                inputPort = device.openInputPort(inputPortNumber)
                Log.d(TAG, "Opening input port $inputPortNumber complete")

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

                currentDevice = device
            }
        }, null)
    }

    fun addHandler(handler: MidiDeviceEventHandler) {
        handlers.add(handler)
        if (inputPort != null && currentDevice != null) {
            handler.onConnect()
        } else {
            handler.onDisconnect()
        }
    }

    fun removeHandler(handler: MidiDeviceEventHandler) {
        handlers.remove(handler)
    }

    fun send(payload: ByteArray, offset: Int, count: Int, timestamp: Long? = null): Boolean {
        return if (inputPort == null) {
            Log.e(TAG, "Trying to send while input port is not opened.")
            false
        } else {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                executor.submit {
                    try {
                        if (timestamp != null) {
                            inputPort?.send(payload, offset, count, timestamp)
                        } else {
                            inputPort?.send(payload, offset, count)
                        }
                    } catch (e: IOException) {
                        Log.e(TAG, "Can't send MIDI message to port: $e")
                    }
                }
            } else {
                handler.post {
                    try {
                        inputPort?.send(payload, offset, count)
                    } catch (e: IOException) {
                        Log.e(TAG, "Can't send MIDI message to port: $e")
                    }
                }
            }

            true
        }
    }

    override fun onDeviceAdded(device: MidiDeviceInfo?) {
        super.onDeviceAdded(device)

        Log.i(TAG, "onDeviceAdded: $device")

        if (device != null && currentDevice != null) {
            initializeDevice(device)
        }
    }

    override fun onDeviceRemoved(device: MidiDeviceInfo?) {
        super.onDeviceRemoved(device)

        Log.i(TAG, "onDeviceRemoved: $device")

        if (device == currentDevice) {
            for (handler in handlers) {
                handler.onDisconnect()
            }
        }

        currentDevice?.close()
        currentDevice = null
        inputPort = null
    }
}