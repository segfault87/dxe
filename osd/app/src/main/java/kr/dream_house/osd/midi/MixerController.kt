package kr.dream_house.osd.midi

import android.util.Log
import androidx.compose.runtime.compositionLocalOf
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.Serializable

@Serializable
data class ChannelData(
    val level: Float = 0.0f,
    val pan: Float = 0.0f,
    val reverb: Float = 0.0f,
    val mute: Boolean = false,
    val eqHighLevel: Float = 0.0f,
    val eqHighFreq: Float = 9490.0f,
    val eqHighQ: Float = 0.5f,
    val eqMidLevel: Float = 0.0f,
    val eqMidFreq: Float = 2080.0f,
    val eqMidQ: Float = 0.5f,
    val eqLowLevel: Float = 0.0f,
    val eqLowFreq: Float = 99.0f,
    val eqLowQ: Float = 0.5f,
)

@Serializable
data class PartialChannelDataUpdate(
    val level: Float? = null,
    val pan: Float? = null,
    val reverb: Float? = null,
    val mute: Boolean? = null,
    val eqHighLevel: Float? = null,
    val eqHighFreq: Float? = null,
    val eqHighQ: Float? = null,
    val eqMidLevel: Float? = null,
    val eqMidFreq: Float? = null,
    val eqMidQ: Float? = null,
    val eqLowLevel: Float? = null,
    val eqLowFreq: Float? = null,
    val eqLowQ: Float? = null,
)

fun ChannelData.updateFrom(update: PartialChannelDataUpdate): ChannelData {
    return ChannelData(
        level = update.level ?: level,
        pan = update.pan ?: pan,
        reverb = update.reverb ?: reverb,
        mute = update.mute ?: mute,
        eqHighLevel = update.eqHighLevel ?: eqHighLevel,
        eqHighFreq = update.eqHighFreq ?: eqHighFreq,
        eqHighQ = update.eqHighQ ?: eqHighQ,
        eqMidLevel = update.eqMidLevel ?: eqMidLevel,
        eqMidFreq = update.eqMidFreq ?: eqMidFreq,
        eqMidQ = update.eqMidQ ?: eqMidQ,
        eqLowLevel = update.eqLowLevel ?: eqLowLevel,
        eqLowFreq = update.eqLowFreq ?: eqLowFreq,
        eqLowQ = update.eqLowQ ?: eqLowQ,
    )
}

private data class TransformedChannelData(
    var level: Byte? = null,
    var pan: Byte? = null,
    var reverb: Byte? = null,
    var mute: Byte? = null,
    var eqHighLevel: Byte? = null,
    var eqHighFreq: Byte? = null,
    var eqHighQ: Byte? = null,
    var eqMidLevel: Byte? = null,
    var eqMidFreq: Byte? = null,
    var eqMidQ: Byte? = null,
    var eqLowLevel: Byte? = null,
    var eqLowFreq: Byte? = null,
    var eqLowQ: Byte? = null,
)

private fun PartialChannelDataUpdate.transform(channel: Int, data: TransformedChannelData, device: MixerDevice): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    level?.let {
        device.translateChannelLevelValue(it)?.let { updatedValue ->
            if (data.level != updatedValue) {
                data.level = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    pan?.let {
        device.translateChannelPanValue(it)?.let { updatedValue ->
            if (data.pan != updatedValue) {
                data.pan = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.PAN,
                    value = updatedValue,
                ))
            }
        }
    }
    reverb?.let {
        device.translateChannelReverbValue(it)?.let { updatedValue ->
            if (data.reverb != updatedValue) {
                data.reverb = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.REVERB,
                    value = updatedValue,
                ))
            }
        }
    }
    mute?.let {
        device.translateChannelMuteValue(it)?.let { updatedValue ->
            if (data.mute != updatedValue) {
                data.mute = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.MUTE,
                    value = updatedValue,
                ))
            }
        }
    }
    eqHighLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (data.eqHighLevel != updatedValue) {
                data.eqHighLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_HIGH_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqHighFreq?.let {
        device.translateChannelThreeBandEqHighFreqValue(it)?.let { updatedValue ->
            if (data.eqHighFreq != updatedValue) {
                data.eqHighFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_HIGH_FREQ,
                    value = updatedValue,
                ))
            }
        }
    }
    eqHighQ?.let {
        device.translateChannelThreeBandEqHighQValue(it)?.let { updatedValue ->
            if (data.eqHighQ != updatedValue) {
                data.eqHighQ = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_HIGH_Q,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (data.eqMidLevel != updatedValue) {
                data.eqMidLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_MID_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidFreq?.let {
        device.translateChannelThreeBandEqMidFreqValue(it)?.let { updatedValue ->
            if (data.eqMidFreq != updatedValue) {
                data.eqMidFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_MID_FREQ,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidQ?.let {
        device.translateChannelThreeBandEqMidQValue(it)?.let { updatedValue ->
            if (data.eqMidQ != updatedValue) {
                data.eqMidQ = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_MID_Q,
                    value = updatedValue,
                ))
            }
        }
    }
    eqLowLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (data.eqLowLevel != updatedValue) {
                data.eqLowLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_LOW_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqLowFreq?.let {
        device.translateChannelThreeBandEqLowFreqValue(it)?.let { updatedValue ->
            if (data.eqLowFreq != updatedValue) {
                data.eqLowFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_LOW_FREQ,
                    value = updatedValue,
                ))
            }
        }
    }
    eqLowQ?.let {
        device.translateChannelThreeBandEqLowQValue(it)?.let { updatedValue ->
            if (data.eqLowQ != updatedValue) {
                data.eqLowQ = updatedValue
                result.add(ControlValue.ChannelValue(
                    channel = channel,
                    control = ChannelControlParameter.EQ_LOW_Q,
                    value = updatedValue,
                ))
            }
        }
    }

    return result
}

private fun ChannelData.transform(device: MixerDevice): TransformedChannelData {
    val level = device.translateChannelLevelValue(level)
    val pan = device.translateChannelPanValue(pan)
    val reverb = device.translateChannelReverbValue(reverb)
    val mute = device.translateChannelMuteValue(mute)
    val eqHighLevel = device.translateChannelEqLevelValue(eqHighLevel)
    val eqMidLevel = device.translateChannelEqLevelValue(eqMidLevel)
    val eqLowLevel = device.translateChannelEqLevelValue(eqLowLevel)
    val eqHighFreq = device.translateChannelThreeBandEqHighFreqValue(eqHighFreq)
    val eqMidFreq = device.translateChannelThreeBandEqMidFreqValue(eqMidFreq)
    val eqLowFreq = device.translateChannelThreeBandEqLowFreqValue(eqLowFreq)
    val eqHighQ = device.translateChannelThreeBandEqHighQValue(eqHighQ)
    val eqMidQ = device.translateChannelThreeBandEqMidQValue(eqMidQ)
    val eqLowQ = device.translateChannelThreeBandEqLowQValue(eqLowQ)

    return TransformedChannelData(
        level = level,
        pan = pan,
        reverb = reverb,
        mute = mute,
        eqHighLevel = eqHighLevel,
        eqMidLevel = eqMidLevel,
        eqLowLevel = eqLowLevel,
        eqHighFreq = eqHighFreq,
        eqMidFreq = eqMidFreq,
        eqLowFreq = eqLowFreq,
        eqHighQ = eqHighQ,
        eqMidQ = eqMidQ,
        eqLowQ = eqLowQ,
    )
}

private fun TransformedChannelData.buildControlPayloads(channel: Int): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    level?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.LEVEL,
            channel = channel,
            value = it,
        ))
    }
    pan?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.PAN,
            channel = channel,
            value = it,
        ))
    }
    reverb?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.REVERB,
            channel = channel,
            value = it,
        ))
    }
    mute?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.MUTE,
            channel = channel,
            value = it,
        ))
    }
    eqHighLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_HIGH_LEVEL,
                channel = channel,
                value = it,
            )
        )
    }
    eqMidLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_LEVEL,
                channel = channel,
                value = it,
            )
        )
    }
    eqLowLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_LOW_LEVEL,
                channel = channel,
                value = it,
            )
        )
    }
    eqHighFreq?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_HIGH_FREQ,
                channel = channel,
                value = it,
            )
        )
    }
    eqMidFreq?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_FREQ,
                channel = channel,
                value = it,
            )
        )
    }
    eqLowFreq?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.EQ_LOW_FREQ,
            channel = channel,
            value = it,
        ))
    }
    eqHighQ?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_HIGH_Q,
                channel = channel,
                value = it,
            )
        )
    }
    eqMidQ?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_Q,
                channel = channel,
                value = it,
            )
        )
    }
    eqLowQ?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_Q,
                channel = channel,
                value = it,
            )
        )
    }

    return result
}

@Serializable
data class GlobalData(
    val masterLevel: Float = 0.0f,
    val monitorLevel: Float = 0.0f,
)

@Serializable
data class PartialGlobalDataUpdate(
    val masterLevel: Float? = null,
    val monitorLevel: Float? = null,
)

fun GlobalData.updateFrom(update: PartialGlobalDataUpdate): GlobalData {
    return GlobalData(
        masterLevel = update.masterLevel ?: masterLevel,
        monitorLevel = update.monitorLevel ?: monitorLevel,
    )
}

private data class TransformedGlobalData(
    var masterLevel: Byte? = null,
    var monitorLevel: Byte? = null,
)

private fun PartialGlobalDataUpdate.transform(data: TransformedGlobalData, device: MixerDevice): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    masterLevel?.let {
        device.translateGlobalMasterLevelValue(it)?.let { updatedValue ->
            if (data.masterLevel != updatedValue) {
                data.masterLevel = updatedValue
                result.add(ControlValue.GlobalValue(
                    control = GlobalControlParameter.MASTER_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    monitorLevel?.let {
        device.translateGlobalMonitorLevelValue(it)?.let { updatedValue ->
            if (data.monitorLevel != updatedValue) {
                data.monitorLevel = updatedValue
                result.add(ControlValue.GlobalValue(
                    control = GlobalControlParameter.MONITOR_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }

    return result
}

private fun GlobalData.transform(device: MixerDevice): TransformedGlobalData {
    val masterLevel = device.translateGlobalMasterLevelValue(masterLevel)
    val monitorLevel = device.translateGlobalMonitorLevelValue(monitorLevel)

    return TransformedGlobalData(
        masterLevel = masterLevel,
        monitorLevel = monitorLevel,
    )
}


private fun TransformedGlobalData.buildControlPayloads(): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    masterLevel?.let {
        result.add(ControlValue.GlobalValue(
            control = GlobalControlParameter.MASTER_LEVEL,
            value = it,
        ))
    }
    monitorLevel?.let {
        result.add(ControlValue.GlobalValue(
            control = GlobalControlParameter.MONITOR_LEVEL,
            value = it,
        ))
    }

    return result
}

@Serializable
data class MixerState(
    val channels: List<ChannelData>,
    val globals: GlobalData,
)

private data class TransformedMixerState(
    var channels: MutableList<TransformedChannelData>,
    var globals: TransformedGlobalData
)

private fun MixerState.transform(device: MixerDevice): TransformedMixerState {
    return TransformedMixerState(
        channels = channels.map { it.transform(device) }.toMutableList(),
        globals = globals.transform(device)
    )
}

private fun TransformedMixerState.buildControlPayloads(): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    channels.forEachIndexed { idx, item ->
        result.addAll(item.buildControlPayloads(idx))
    }
    result.addAll(globals.buildControlPayloads())

    return result
}

class MixerController(
    private val midiDeviceManager: MidiDeviceManager,
    private val device: MixerDevice,
    defaultInitialChannelStates: List<ChannelData>? = null,
    defaultInitialGlobalStates: GlobalData? = null,
) : MidiDeviceEventHandler {

    companion object {
        private const val TAG = "MixerController"
    }

    private var initialChannelStates: List<ChannelData> = defaultInitialChannelStates ?: device.channelNames.map {
        ChannelData()
    }.toList()
    private var initialGlobalStates: GlobalData = defaultInitialGlobalStates ?: GlobalData()

    private val _state = MutableStateFlow(MixerState(
        channels = initialChannelStates,
        globals = initialGlobalStates
    ))
    val state = _state.asStateFlow()

    private var transformedState = _state.value.transform(device)

    private val _isConnected = MutableStateFlow(false)
    val isConnected = _isConnected.asStateFlow()

    val channels: Array<String>
        get() = device.channelNames

    fun attach() {
        midiDeviceManager.addHandler(this)
    }

    fun detach() {
        midiDeviceManager.removeHandler(this)
    }

    fun updateValues(channel: Int, update: PartialChannelDataUpdate): Boolean {
        _state.update { currentState ->
            val updatedChannelData = currentState.channels.mapIndexed { idx, item ->
                if (idx == channel) {
                    item.updateFrom(update)
                } else {
                    item
                }
            }
            currentState.copy(channels = updatedChannelData)
        }

        val updates = update.transform(channel, transformedState.channels[channel], device)
        return pushUpdates(updates)
    }

    fun updateValues(update: PartialGlobalDataUpdate): Boolean {
        _state.update { currentState ->
            currentState.copy(globals = currentState.globals.updateFrom(update))
        }

        val updates = update.transform(transformedState.globals, device)
        return pushUpdates(updates)
    }

    private fun pushUpdates(updates: List<ControlValue>): Boolean {
        return if (updates.isNotEmpty()) {
            val size = updates.sumOf { device.getCCPayloadSizeHint(it) }
            val payload = ByteArray(size)
            var offset = 0
            for (update in updates) {
                offset += device.buildCCPayload(update, payload, offset)
            }
            midiDeviceManager.send(payload, 0, size)
        } else {
            false
        }
    }

    private fun updateState(state: ControlValue): Boolean {
        when (state) {
            is ControlValue.ChannelValue -> {
                val channel = state.channel
                val value = state.value
                val channelData = _state.value.channels[channel]
                val transformedData = transformedState.channels[channel]
                val updatedData = when (state.control) {
                    ChannelControlParameter.LEVEL -> {
                        val localValue = device.translateRemoteChannelLevelValue(value)
                        if (localValue != null && transformedData.level != value) {
                            transformedData.level = value
                            channelData.copy(level = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.PAN -> {
                        val localValue = device.translateRemoteChannelPanValue(value)
                        if (localValue != null && transformedData.pan != value) {
                            transformedData.pan = value
                            channelData.copy(pan = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.REVERB -> {
                        val localValue = device.translateRemoteChannelReverbValue(value)
                        if (localValue != null && transformedData.reverb != value) {
                            transformedData.reverb = value
                            channelData.copy(reverb = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.MUTE -> {
                        val localValue = device.translateRemoteChannelMuteValue(value)
                        if (localValue != null && transformedData.mute != value) {
                            transformedData.mute = value
                            channelData.copy(mute = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_HIGH_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && transformedData.eqHighLevel != value) {
                            transformedData.eqHighLevel = value
                            channelData.copy(eqHighLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_HIGH_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqHighFreqValue(value)
                        if (localValue != null && transformedData.eqHighFreq != value) {
                            transformedData.eqHighFreq = value
                            channelData.copy(eqHighFreq = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_HIGH_Q -> {
                        val localValue = device.translateRemoteChannelThreeBandEqHighQValue(value)
                        if (localValue != null && transformedData.eqHighQ != value) {
                            transformedData.eqHighQ = value
                            channelData.copy(eqHighQ = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && transformedData.eqMidLevel != value) {
                            transformedData.eqMidLevel = value
                            channelData.copy(eqMidLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqMidFreqValue(value)
                        if (localValue != null && transformedData.eqMidFreq != value) {
                            transformedData.eqMidFreq = value
                            channelData.copy(eqMidFreq = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_Q -> {
                        val localValue = device.translateRemoteChannelThreeBandEqMidQValue(value)
                        if (localValue != null && transformedData.eqMidQ != value) {
                            transformedData.eqMidQ = value
                            channelData.copy(eqMidQ = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_LOW_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && transformedData.eqLowLevel != value) {
                            transformedData.eqLowLevel = value
                            channelData.copy(eqLowLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_LOW_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqLowFreqValue(value)
                        if (localValue != null && transformedData.eqLowFreq != value) {
                            transformedData.eqLowFreq = value
                            channelData.copy(eqLowFreq = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_LOW_Q -> {
                        val localValue = device.translateRemoteChannelThreeBandEqLowQValue(value)
                        if (localValue != null && transformedData.eqLowQ != value) {
                            transformedData.eqLowQ = value
                            channelData.copy(eqLowQ = localValue)
                        } else {
                            null
                        }
                    }
                }

                return if (updatedData != null) {
                    _state.update { currentState ->
                        val updatedChannelData = currentState.channels.mapIndexed { idx, item ->
                            if (idx == channel) {
                                updatedData
                            } else {
                                item
                            }
                        }
                        currentState.copy(channels = updatedChannelData)
                    }
                    true
                } else {
                    false
                }
            }
            is ControlValue.GlobalValue -> {
                val globalData = _state.value.globals
                val transformedData = transformedState.globals
                val value = state.value
                val updatedData = when (state.control) {
                    GlobalControlParameter.MASTER_LEVEL -> {
                        val localValue = device.translateRemoteGlobalMasterLevelValue(value)
                        if (localValue != null && transformedData.masterLevel != value) {
                            transformedData.masterLevel = value
                            globalData.copy(masterLevel = localValue)
                        } else {
                            null
                        }
                    }
                    GlobalControlParameter.MONITOR_LEVEL -> {
                        val localValue = device.translateRemoteGlobalMonitorLevelValue(value)
                        if (localValue != null && transformedData.monitorLevel != value) {
                            transformedData.monitorLevel = value
                            globalData.copy(monitorLevel = localValue)
                        } else {
                            null
                        }
                    }
                }

                return if (updatedData != null) {
                    _state.update { currentState ->
                        currentState.copy(globals = updatedData)
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    fun updateInitialChannelStates(channelStates: List<ChannelData>, globalStates: GlobalData) {
        // Truncate to configured channel count
        val newChannelStates = channelStates.take(channels.size).toMutableList()
        if (newChannelStates.size < channels.size) {
            newChannelStates.addAll(List(channels.size - newChannelStates.size) { ChannelData() })
        }
        initialChannelStates = newChannelStates
        initialGlobalStates = globalStates

        resetStates()
    }

    fun resetStates() {
        _state.update {
            MixerState(
                channels = initialChannelStates,
                globals = initialGlobalStates
            )
        }

        transformedState = _state.value.transform(device)

        val controlValues = transformedState.buildControlPayloads()
        sendBulkPayloads(device.initializeState(controlValues))
    }

    private fun sendBulkPayloads(payloads: List<ByteArray>) {
        var timestamp = System.nanoTime()
        for (payload in payloads) {
            // Sometimes mixer can't accept bulk messages at once and we have to schedule it by 10 msecs interval
            midiDeviceManager.send(payload, 0, payload.size, timestamp)
            timestamp += 10000000
        }
    }

    override fun onReceive(payload: ByteArray, offset: Int, count: Int) {
        device.parseCCPayload(payload, offset, count)?.let {
            updateState(it)
        }
    }

    override fun onConnect() {
        _isConnected.update { true }

        val controlValues = transformedState.buildControlPayloads()
        Log.d(TAG, "Connected to mixer. initializing values...")
        sendBulkPayloads(device.initializeState(controlValues))
    }

    override fun onDisconnect() {
        _isConnected.update { false }
    }

}

val LocalMixerController = compositionLocalOf<MixerController?> { null }