package kr.dream_house.osd.midi

enum class MixerCapability {
    CHANNEL_LEVEL,
    CHANNEL_PAN,
    CHANNEL_REVERB,
    CHANNEL_MUTE,
    CHANNEL_THREE_BAND_EQ_LEVEL,
    CHANNEL_THREE_BAND_EQ_HIGH_FREQ,
    CHANNEL_THREE_BAND_EQ_MID_FREQ,
    CHANNEL_THREE_BAND_EQ_LOW_FREQ,
    CHANNEL_THREE_BAND_EQ_LOW_Q,
    CHANNEL_THREE_BAND_EQ_MID_Q,
    CHANNEL_THREE_BAND_EQ_HIGH_Q,
    GLOBAL_MASTER_LEVEL,
    GLOBAL_MONITOR_LEVEL,
}

enum class ChannelControlParameter {
    LEVEL,
    PAN,
    REVERB,
    MUTE,
    EQ_LOW_FREQ,
    EQ_LOW_LEVEL,
    EQ_LOW_Q,
    EQ_MID_FREQ,
    EQ_MID_LEVEL,
    EQ_MID_Q,
    EQ_HIGH_FREQ,
    EQ_HIGH_LEVEL,
    EQ_HIGH_Q,
}

enum class GlobalControlParameter {
    MASTER_LEVEL,
    MONITOR_LEVEL,
}

sealed class ControlValue {
    data class ChannelValue(
        val control: ChannelControlParameter,
        val channel: Int,
        val value: Byte,
    ) : ControlValue()

    data class GlobalValue(
        val control: GlobalControlParameter,
        val value: Byte,
    ) : ControlValue()
}

interface MixerDevice {
    val channelNames: Array<String>

    fun translateChannelLevelValue(level: Float): Byte? = null
    fun translateRemoteChannelLevelValue(value: Byte): Float? = null
    fun translateChannelPanValue(pan: Float): Byte? = null
    fun translateRemoteChannelPanValue(value: Byte): Float? = null
    fun translateChannelReverbValue(reverb: Float): Byte? = null
    fun translateRemoteChannelReverbValue(value: Byte): Float? = null
    fun translateChannelMuteValue(mute: Boolean): Byte? = null
    fun translateRemoteChannelMuteValue(value: Byte): Boolean? = null
    fun translateChannelEqLevelValue(level: Float): Byte? = null
    fun translateRemoteChannelEqLevelValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqHighFreqValue(freq: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqHighFreqValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqMidFreqValue(freq: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqMidFreqValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqLowFreqValue(freq: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqLowFreqValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqHighQValue(q: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqHighQValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqMidQValue(q: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqMidQValue(value: Byte): Float? = null
    fun translateChannelThreeBandEqLowQValue(q: Float): Byte? = null
    fun translateRemoteChannelThreeBandEqLowQValue(value: Byte): Float? = null

    fun translateGlobalMasterLevelValue(level: Float): Byte? = null
    fun translateRemoteGlobalMasterLevelValue(value: Byte): Float? = null
    fun translateGlobalMonitorLevelValue(level: Float): Byte? = null
    fun translateRemoteGlobalMonitorLevelValue(value: Byte): Float? = null

    fun initializeState(initialStates: List<ControlValue>): List<ByteArray>
    fun parseCCPayload(packet: ByteArray, offset: Int, size: Int): ControlValue?
    fun buildCCPayload(value: ControlValue, output: ByteArray, offset: Int): Int
    fun getCCPayloadSizeHint(value: ControlValue): Int

    fun flowControlMilliseconds(): Long
    fun maxPayloadInBatch(): Int
}

fun MixerDevice.queryCapability(capability: MixerCapability): Boolean {
    return when (capability) {
        MixerCapability.CHANNEL_LEVEL -> translateChannelLevelValue(0.0f) != null
        MixerCapability.CHANNEL_PAN -> translateChannelPanValue(0.0f) != null
        MixerCapability.CHANNEL_REVERB -> translateChannelReverbValue(0.0f) != null
        MixerCapability.CHANNEL_MUTE -> translateChannelMuteValue(false) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_LEVEL -> translateChannelEqLevelValue(0.0f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_HIGH_FREQ -> translateChannelThreeBandEqHighFreqValue(10000.0f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_MID_FREQ -> translateChannelThreeBandEqMidFreqValue(2000.0f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_LOW_FREQ -> translateChannelThreeBandEqLowFreqValue(100.0f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_HIGH_Q -> translateChannelThreeBandEqHighQValue(0.5f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_MID_Q -> translateChannelThreeBandEqMidQValue(0.5f) != null
        MixerCapability.CHANNEL_THREE_BAND_EQ_LOW_Q -> translateChannelThreeBandEqLowQValue(0.5f) != null
        MixerCapability.GLOBAL_MASTER_LEVEL -> translateGlobalMasterLevelValue(0.0f) != null
        MixerCapability.GLOBAL_MONITOR_LEVEL -> translateGlobalMonitorLevelValue(0.0f) != null
    }
}