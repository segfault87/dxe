package kr.dream_house.osd.midi

import android.util.Log
import androidx.compose.runtime.compositionLocalOf
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.Serializable
import kr.dream_house.osd.MixerChannelId
import kr.dream_house.osd.entities.MixerPresets
import kr.dream_house.osd.entities.PartialChannelDataUpdate
import kr.dream_house.osd.entities.PartialGlobalDataUpdate
import kotlin.concurrent.atomics.AtomicBoolean
import kotlin.concurrent.atomics.ExperimentalAtomicApi

@Serializable
data class ChannelData(
    val level: Float = Float.NEGATIVE_INFINITY,
    val pan: Float = 0.0f,
    val reverb: Float = Float.NEGATIVE_INFINITY,
    val mute: Boolean = false,
    val eqHighLevel: Float = 0.0f,
    val eqHighFreq: Float = 9490.0f,
    val eqMidLevel: Float = 0.0f,
    val eqMidFreq: Float = 2080.0f,
    val eqMidQ: Float = 0.5f,
    val eqLowLevel: Float = 0.0f,
    val eqLowFreq: Float = 99.0f,
)

fun ChannelData.snapshot(): PartialChannelDataUpdate {
    return PartialChannelDataUpdate(
        level = level,
        pan = pan,
        reverb = reverb,
        mute = mute,
        eqHighLevel = eqHighLevel,
        eqHighFreq = eqHighFreq,
        eqMidLevel = eqMidLevel,
        eqMidFreq = eqMidFreq,
        eqMidQ = eqMidQ,
        eqLowLevel = eqLowLevel,
        eqLowFreq = eqLowFreq,
    )
}

fun ChannelData.updateFrom(update: PartialChannelDataUpdate): ChannelData {
    return ChannelData(
        level = update.level ?: level,
        pan = update.pan ?: pan,
        reverb = update.reverb ?: reverb,
        mute = update.mute ?: mute,
        eqHighLevel = update.eqHighLevel ?: eqHighLevel,
        eqHighFreq = update.eqHighFreq ?: eqHighFreq,
        eqMidLevel = update.eqMidLevel ?: eqMidLevel,
        eqMidFreq = update.eqMidFreq ?: eqMidFreq,
        eqMidQ = update.eqMidQ ?: eqMidQ,
        eqLowLevel = update.eqLowLevel ?: eqLowLevel,
        eqLowFreq = update.eqLowFreq ?: eqLowFreq,
    )
}

private class TransformedChannelData(
    var level: ByteArray? = null,
    var pan: ByteArray? = null,
    var reverb: ByteArray? = null,
    var mute: ByteArray? = null,
    var eqHighLevel: ByteArray? = null,
    var eqHighFreq: ByteArray? = null,
    var eqMidLevel: ByteArray? = null,
    var eqMidFreq: ByteArray? = null,
    var eqMidQ: ByteArray? = null,
    var eqLowLevel: ByteArray? = null,
    var eqLowFreq: ByteArray? = null,
)

private fun PartialChannelDataUpdate.transform(device: MixerDevice, channel: MixerChannelConfig, data: TransformedChannelData): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    level?.let {
        device.translateChannelLevelValue(it)?.let { updatedValue ->
            if (!data.level.contentEquals(updatedValue)) {
                data.level = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    pan?.let {
        device.translateChannelPanValue(it)?.let { updatedValue ->
            if (!data.pan.contentEquals(updatedValue)) {
                data.pan = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.PAN,
                    value = updatedValue,
                ))
            }
        }
    }
    reverb?.let {
        device.translateChannelReverbValue(it)?.let { updatedValue ->
            if (!data.reverb.contentEquals(updatedValue)) {
                data.reverb = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.REVERB,
                    value = updatedValue,
                ))
            }
        }
    }
    mute?.let {
        device.translateChannelMuteValue(it)?.let { updatedValue ->
            if (!data.mute.contentEquals(updatedValue)) {
                data.mute = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.MUTE,
                    value = updatedValue,
                ))
            }
        }
    }
    eqHighLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (!data.eqHighLevel.contentEquals(updatedValue)) {
                data.eqHighLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_HIGH_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqHighFreq?.let {
        device.translateChannelThreeBandEqHighFreqValue(it)?.let { updatedValue ->
            if (!data.eqHighFreq.contentEquals(updatedValue)) {
                data.eqHighFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_HIGH_FREQ,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (!data.eqMidLevel.contentEquals(updatedValue)) {
                data.eqMidLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_MID_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidFreq?.let {
        device.translateChannelThreeBandEqMidFreqValue(it)?.let { updatedValue ->
            if (!data.eqMidFreq.contentEquals(updatedValue)) {
                data.eqMidFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_MID_FREQ,
                    value = updatedValue,
                ))
            }
        }
    }
    eqMidQ?.let {
        device.translateChannelThreeBandEqMidQValue(it)?.let { updatedValue ->
            if (!data.eqMidQ.contentEquals(updatedValue)) {
                data.eqMidQ = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_MID_Q,
                    value = updatedValue,
                ))
            }
        }
    }
    eqLowLevel?.let {
        device.translateChannelEqLevelValue(it)?.let { updatedValue ->
            if (!data.eqLowLevel.contentEquals(updatedValue)) {
                data.eqLowLevel = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_LOW_LEVEL,
                    value = updatedValue,
                ))
            }
        }
    }
    eqLowFreq?.let {
        device.translateChannelThreeBandEqLowFreqValue(it)?.let { updatedValue ->
            if (!data.eqLowFreq.contentEquals(updatedValue)) {
                data.eqLowFreq = updatedValue
                result.add(ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.EQ_LOW_FREQ,
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
    val eqMidQ = device.translateChannelThreeBandEqMidQValue(eqMidQ)

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
        eqMidQ = eqMidQ,
    )
}

private fun TransformedChannelData.buildControlPayloads(channel: MixerChannelConfig): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    level?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.LEVEL,
            channelId = channel.id,
            value = it,
        ))
    }
    pan?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.PAN,
            channelId = channel.id,
            value = it,
        ))
    }
    reverb?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.REVERB,
            channelId = channel.id,
            value = it,
        ))
    }
    mute?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.MUTE,
            channelId = channel.id,
            value = it,
        ))
    }
    eqHighLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_HIGH_LEVEL,
                channelId = channel.id,
                value = it,
            )
        )
    }
    eqMidLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_LEVEL,
                channelId = channel.id,
                value = it,
            )
        )
    }
    eqLowLevel?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_LOW_LEVEL,
                channelId = channel.id,
                value = it,
            )
        )
    }
    eqHighFreq?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_HIGH_FREQ,
                channelId = channel.id,
                value = it,
            )
        )
    }
    eqMidFreq?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_FREQ,
                channelId = channel.id,
                value = it,
            )
        )
    }
    eqLowFreq?.let {
        result.add(ControlValue.ChannelValue(
            control = ChannelControlParameter.EQ_LOW_FREQ,
            channelId = channel.id,
            value = it,
        ))
    }
    eqMidQ?.let {
        result.add(
            ControlValue.ChannelValue(
                control = ChannelControlParameter.EQ_MID_Q,
                channelId = channel.id,
                value = it,
            )
        )
    }

    return result
}

@Serializable
data class GlobalData(
    val masterLevel: Float = Float.NEGATIVE_INFINITY,
    val monitorLevel: Float = Float.NEGATIVE_INFINITY,
)

private fun GlobalData.snapshot(): PartialGlobalDataUpdate {
    return PartialGlobalDataUpdate(
        masterLevel = masterLevel,
        monitorLevel = monitorLevel,
    )
}

fun GlobalData.updateFrom(update: PartialGlobalDataUpdate): GlobalData {
    return GlobalData(
        masterLevel = update.masterLevel ?: masterLevel,
        monitorLevel = update.monitorLevel ?: monitorLevel,
    )
}

private class TransformedGlobalData(
    var masterLevel: ByteArray? = null,
    var monitorLevel: ByteArray? = null,
)

private fun PartialGlobalDataUpdate.transform(device: MixerDevice, data: TransformedGlobalData): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    masterLevel?.let {
        device.translateGlobalMasterLevelValue(it)?.let { updatedValue ->
            if (!data.masterLevel.contentEquals(updatedValue)) {
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
            if (!data.monitorLevel.contentEquals(updatedValue)) {
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
    val channels: Map<MixerChannelId, ChannelData> = emptyMap(),
    val globals: GlobalData = GlobalData(),
)

private data class TransformedMixerState(
    var channels: MutableMap<MixerChannelId, TransformedChannelData>,
    var globals: TransformedGlobalData
)

private fun MixerState.transform(device: MixerDevice): TransformedMixerState {
    return TransformedMixerState(
        channels = channels.map { (key, value) -> key to value.transform(device) }.toMap().toMutableMap(),
        globals = globals.transform(device)
    )
}

private fun TransformedMixerState.buildControlPayloads(config: MixerConfigurations): List<ControlValue> {
    val result = mutableListOf<ControlValue>()

    channels.forEach { (key, value) ->
        config.channelsById[key]?.let {
            result.addAll(value.buildControlPayloads(it))
        }
    }
    result.addAll(globals.buildControlPayloads())

    return result
}

// TODO: make it dynamically configurable
private fun getMixerConfigurations(device: MixerDevice): MixerConfigurations {
    return DefaultConfig.MIXER_CONFIGURATIONS.get(device.spec.identifier) ?: MixerConfigurations(channels = emptyList())
}

class MixerController(private val midiDeviceManager: MidiDeviceManager) : MidiDeviceEventHandler {

    companion object {
        private const val TAG = "MixerController"
    }

    private var initialChannelStates: Map<MixerChannelId, ChannelData> = emptyMap()
    private var initialGlobalStates = GlobalData()

    private var mixerDevice: MixerDevice? = null
    private val _mixerConfigurations = MutableStateFlow<MixerConfigurations?>(null)
    val mixerConfigurations = _mixerConfigurations.asStateFlow()

    private val _state = MutableStateFlow(MixerState(
        channels = initialChannelStates,
        globals = initialGlobalStates
    ))
    val state = _state.asStateFlow()

    private var transformedState: TransformedMixerState? = null

    private val _isConnected = MutableStateFlow(false)
    val isConnected = _isConnected.asStateFlow()

    private val _capabilities = MutableStateFlow<Set<MixerCapability>>(emptySet())
    val capabilities = _capabilities.asStateFlow()

    init {
        midiDeviceManager.currentMixer?.let {
            onConnect(it)
        }
    }

    fun attach() {
        midiDeviceManager.addHandler(this)
    }

    fun detach() {
        midiDeviceManager.removeHandler(this)
    }

    fun updateValues(channelId: MixerChannelId, update: PartialChannelDataUpdate) {
        val device = mixerDevice ?: return

        mixerConfigurations.value?.let { config ->
            val channel = config.channelsById[channelId] ?: return@let

            _state.update { currentState ->
                val updatedChannelData = currentState.channels.map { (key, item) ->
                    if (key == channelId) {
                        key to item.updateFrom(update)
                    } else {
                        key to item
                    }
                }.toMap()
                currentState.copy(channels = updatedChannelData)
            }

            val transformedChannel = transformedState!!.channels.getOrPut(channelId) { TransformedChannelData() }

            val updates = update.transform(device, channel, transformedChannel)
            pushUpdates(updates)
        }
    }

    fun updateValues(update: PartialGlobalDataUpdate) {
        mixerDevice?.let { device ->
            _state.update { currentState ->
                currentState.copy(globals = currentState.globals.updateFrom(update))
            }

            val updates = update.transform(device, transformedState!!.globals)
            pushUpdates(updates)
        }
    }

    fun snapshot(): MixerPresets {
        return MixerPresets(
            channels = _state.value.channels.map { (key, value) -> key to value.snapshot() }.toMap(),
            globals = _state.value.globals.snapshot(),
        )
    }

    private fun pushUpdates(updates: List<ControlValue>) {
        val config = mixerConfigurations.value ?: return
        val device = mixerDevice ?: return

        when {
            updates.isEmpty() -> {}
            updates.size <= device.maxPayloadInBatch() -> {
                // Post small updates at once
                val size = updates.sumOf { device.getMidiPayloadSizeHint(config, it) }
                val payload = ByteArray(size)
                var offset = 0
                for (update in updates) {
                    offset += device.buildMidiPayload(config, update, payload, offset)
                }
                midiDeviceManager.send(payload, 0, size)
            }
            else -> {
                // Send in chunks in deferred if the payload is too big
                val payloads = mutableListOf<ByteArray>()
                for (update in updates) {
                    val payload = ByteArray(device.getMidiPayloadSizeHint(config, update))
                    device.buildMidiPayload(config, update, payload, 0)
                    payloads.add(payload)
                }
                sendBulkPayloads(payloads)
            }
        }
    }

    private fun updateState(state: ControlValue): Boolean {
        val device = mixerDevice ?: return false
        val transformedState = transformedState ?: return false

        when (state) {
            is ControlValue.ChannelValue -> {
                val channelId = state.channelId
                val value = state.value
                val channelData = _state.value.channels[channelId] ?: return false
                val transformedData = transformedState.channels[channelId] ?: return false
                val updatedData = when (state.control) {
                    ChannelControlParameter.LEVEL -> {
                        val localValue = device.translateRemoteChannelLevelValue(value)
                        if (localValue != null && !transformedData.level.contentEquals(value)) {
                            transformedData.level = value
                            channelData.copy(level = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.PAN -> {
                        val localValue = device.translateRemoteChannelPanValue(value)
                        if (localValue != null && !transformedData.pan.contentEquals(value)) {
                            transformedData.pan = value
                            channelData.copy(pan = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.REVERB -> {
                        val localValue = device.translateRemoteChannelReverbValue(value)
                        if (localValue != null && !transformedData.reverb.contentEquals(value)) {
                            transformedData.reverb = value
                            channelData.copy(reverb = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.MUTE -> {
                        val localValue = device.translateRemoteChannelMuteValue(value)
                        if (localValue != null && !transformedData.mute.contentEquals(value)) {
                            transformedData.mute = value
                            channelData.copy(mute = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_HIGH_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && !transformedData.eqHighLevel.contentEquals(value)) {
                            transformedData.eqHighLevel = value
                            channelData.copy(eqHighLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_HIGH_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqHighFreqValue(value)
                        if (localValue != null && !transformedData.eqHighFreq.contentEquals(value)) {
                            transformedData.eqHighFreq = value
                            channelData.copy(eqHighFreq = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && !transformedData.eqMidLevel.contentEquals(value)) {
                            transformedData.eqMidLevel = value
                            channelData.copy(eqMidLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqMidFreqValue(value)
                        if (localValue != null && !transformedData.eqMidFreq.contentEquals(value)) {
                            transformedData.eqMidFreq = value
                            channelData.copy(eqMidFreq = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_MID_Q -> {
                        val localValue = device.translateRemoteChannelThreeBandEqMidQValue(value)
                        if (localValue != null && !transformedData.eqMidQ.contentEquals(value)) {
                            transformedData.eqMidQ = value
                            channelData.copy(eqMidQ = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_LOW_LEVEL -> {
                        val localValue = device.translateRemoteChannelEqLevelValue(value)
                        if (localValue != null && !transformedData.eqLowLevel.contentEquals(value)) {
                            transformedData.eqLowLevel = value
                            channelData.copy(eqLowLevel = localValue)
                        } else {
                            null
                        }
                    }
                    ChannelControlParameter.EQ_LOW_FREQ -> {
                        val localValue = device.translateRemoteChannelThreeBandEqLowFreqValue(value)
                        if (localValue != null && !transformedData.eqLowFreq.contentEquals(value)) {
                            transformedData.eqLowFreq = value
                            channelData.copy(eqLowFreq = localValue)
                        } else {
                            null
                        }
                    }
                }

                return if (updatedData != null) {
                    _state.update { currentState ->
                        val updatedChannelData = currentState.channels.map { (key, item) ->
                            if (key == channelId) {
                                key to updatedData
                            } else {
                                key to item
                            }
                        }.toMap()
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
                        if (localValue != null && !transformedData.masterLevel.contentEquals(value)) {
                            transformedData.masterLevel = value
                            globalData.copy(masterLevel = localValue)
                        } else {
                            null
                        }
                    }
                    GlobalControlParameter.MONITOR_LEVEL -> {
                        val localValue = device.translateRemoteGlobalMonitorLevelValue(value)
                        if (localValue != null && !transformedData.monitorLevel.contentEquals(value)) {
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

    @OptIn(ExperimentalAtomicApi::class)
    suspend fun checkMixerLiveliness() {
        val flag = AtomicBoolean(true)

        val onChecked: () -> Unit = {
            flag.store(false)
        }

        midiDeviceManager.addIdentityRequestCallback(onChecked)
        var retries = 0
        while (flag.load()) {
            midiDeviceManager.sendIdentityRequest()
            delay(1000)
            retries += 1
            if (retries > 5) {
                Log.w(TAG, "Mixer is not responding. Setting state to false")
                _isConnected.update {
                    false
                }
            }
        }

        midiDeviceManager.removeIdentityRequestCallback(onChecked)
        _isConnected.update {
            true
        }
    }

    suspend fun updateInitialChannelStates(channelStates: Map<MixerChannelId, ChannelData>, globalStates: GlobalData) {
        initialChannelStates = channelStates
        initialGlobalStates = globalStates

        resetStates()
    }

    suspend fun resetStates() {
        val device = mixerDevice ?: return
        val config = mixerConfigurations.value ?: return

        _state.update {
            MixerState(
                channels = initialChannelStates,
                globals = initialGlobalStates
            )
        }

        transformedState = _state.value.transform(device)

        // checkMixerLiveliness()

        val controlValues = transformedState!!.buildControlPayloads(config)
        sendBulkPayloads(device.initializeState(config, controlValues))
    }

    private fun sendBulkPayloads(payloads: List<ByteArray>) {
        mixerDevice?.let { device ->
            // Sometimes mixer can't accept bulk messages at once and we have to schedule it by given msecs interval
            midiDeviceManager.enqueueBulkPayloads(payloads, device.flowControlMilliseconds())
        }
    }

    override fun onReceive(payload: ByteArray, offset: Int, count: Int) {
        mixerConfigurations.value?.let { config ->
            mixerDevice?.let { device ->
                device.parseMidiPayload(config, payload, offset, count)?.let {
                    updateState(it)
                }
            }
        }
    }

    override fun onConnect(mixer: MixerDevice) {
        _isConnected.update { true }

        Log.i(TAG, "Mixer connected: $mixer")

        val config = getMixerConfigurations(mixer)

        mixerDevice = mixer
        transformedState = TransformedMixerState(
            channels = config.channels.associate { it.id to TransformedChannelData() }.toMutableMap(),
            globals = TransformedGlobalData(),
        )
        _mixerConfigurations.update {
            config
        }
        _capabilities.update {
            MixerCapability.entries.filter { mixer.queryCapability(it) }.toSet()
        }
        _state.update {
            MixerState(
                channels = config.channels.associate { it.id to ChannelData() },
                globals = GlobalData(),
            )
        }

        val controlValues = transformedState!!.buildControlPayloads(config)
        Log.i(TAG, "Connected to mixer. initializing values...")
        sendBulkPayloads(mixer.initializeState(config, controlValues))
    }

    override fun onDisconnect() {
        Log.w(TAG, "Mixer disconnected.")

        mixerDevice = null
        transformedState = null

        _capabilities.update { emptySet() }
        _mixerConfigurations.update { null }
        _isConnected.update { false }
        _state.update {
            MixerState(
                channels = emptyMap(),
                globals = GlobalData(),
            )
        }
    }

}

val LocalMixerController = compositionLocalOf<MixerController?> { null }