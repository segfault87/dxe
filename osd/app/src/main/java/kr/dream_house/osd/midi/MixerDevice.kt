package kr.dream_house.osd.midi

import android.os.Bundle
import kr.dream_house.osd.MixerChannelId

enum class MixerCapability {
    CHANNEL_LEVEL,
    CHANNEL_PAN,
    CHANNEL_REVERB,
    CHANNEL_MUTE,
    CHANNEL_THREE_BAND_EQ_LEVEL,
    CHANNEL_THREE_BAND_EQ_HIGH_FREQ,
    CHANNEL_THREE_BAND_EQ_MID_FREQ,
    CHANNEL_THREE_BAND_EQ_LOW_FREQ,
    CHANNEL_THREE_BAND_EQ_MID_Q,
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
    EQ_MID_FREQ,
    EQ_MID_LEVEL,
    EQ_MID_Q,
    EQ_HIGH_FREQ,
    EQ_HIGH_LEVEL,
}

enum class GlobalControlParameter {
    MASTER_LEVEL,
    MONITOR_LEVEL,
}

sealed class ControlValue {
    data class ChannelValue(
        val control: ChannelControlParameter,
        val channelId: MixerChannelId,
        val value: ByteArray,
    ) : ControlValue() {
        override fun equals(other: Any?): Boolean {
            val other = other as? ChannelValue ?: return false

            return control == other.control && channelId == other.channelId && value.contentEquals(other.value)
        }

        override fun hashCode(): Int {
            return control.hashCode() xor channelId.hashCode() xor value.contentHashCode()
        }
    }

    data class GlobalValue(
        val control: GlobalControlParameter,
        val value: ByteArray,
    ) : ControlValue() {
        override fun equals(other: Any?): Boolean {
            val other = other as? GlobalValue ?: return false

            return control == other.control && value.contentEquals(other.value)
        }

        override fun hashCode(): Int {
            return control.hashCode() xor value.contentHashCode()
        }
    }
}

interface MixerDevice {
    val spec: MidiMixerSpec

    fun translateChannelLevelValue(level: Float): ByteArray? = null
    fun translateRemoteChannelLevelValue(value: ByteArray): Float? = null
    fun translateChannelPanValue(pan: Float): ByteArray? = null
    fun translateRemoteChannelPanValue(value: ByteArray): Float? = null
    fun translateChannelReverbValue(reverb: Float): ByteArray? = null
    fun translateRemoteChannelReverbValue(value: ByteArray): Float? = null
    fun translateChannelMuteValue(mute: Boolean): ByteArray? = null
    fun translateRemoteChannelMuteValue(value: ByteArray): Boolean? = null
    fun translateChannelEqLevelValue(level: Float): ByteArray? = null
    fun translateRemoteChannelEqLevelValue(value: ByteArray): Float? = null
    fun translateChannelThreeBandEqHighFreqValue(freq: Float): ByteArray? = null
    fun translateRemoteChannelThreeBandEqHighFreqValue(value: ByteArray): Float? = null
    fun translateChannelThreeBandEqMidFreqValue(freq: Float): ByteArray? = null
    fun translateRemoteChannelThreeBandEqMidFreqValue(value: ByteArray): Float? = null
    fun translateChannelThreeBandEqLowFreqValue(freq: Float): ByteArray? = null
    fun translateRemoteChannelThreeBandEqLowFreqValue(value: ByteArray): Float? = null
    fun translateChannelThreeBandEqMidQValue(q: Float): ByteArray? = null
    fun translateRemoteChannelThreeBandEqMidQValue(value: ByteArray): Float? = null

    fun translateGlobalMasterLevelValue(level: Float): ByteArray? = null
    fun translateRemoteGlobalMasterLevelValue(value: ByteArray): Float? = null
    fun translateGlobalMonitorLevelValue(level: Float): ByteArray? = null
    fun translateRemoteGlobalMonitorLevelValue(value: ByteArray): Float? = null

    fun initializeState(config: MixerConfigurations, initialStates: List<ControlValue>): List<ByteArray>
    fun parseMidiPayload(config: MixerConfigurations, packet: ByteArray, offset: Int, size: Int): ControlValue?
    fun buildMidiPayload(config: MixerConfigurations, value: ControlValue, output: ByteArray, offset: Int): Int
    fun getMidiPayloadSizeHint(config: MixerConfigurations, value: ControlValue): Int

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
        MixerCapability.CHANNEL_THREE_BAND_EQ_MID_Q -> translateChannelThreeBandEqMidQValue(0.5f) != null
        MixerCapability.GLOBAL_MASTER_LEVEL -> translateGlobalMasterLevelValue(0.0f) != null
        MixerCapability.GLOBAL_MONITOR_LEVEL -> translateGlobalMonitorLevelValue(0.0f) != null
    }
}

class MidiDeviceIdentifier(
    val manufacturerId: ByteArray,
    val deviceFamily: ByteArray,
    val familyMember: ByteArray,
    val revision: ByteArray,
)

interface MidiMixerSpec {
    val identifier: String
    fun probe(properties: Bundle): MixerDevice? = null
    fun probe(identifier: MidiDeviceIdentifier): MixerDevice? = null
}