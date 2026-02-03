package kr.dream_house.osd.entities

import kotlinx.serialization.Serializable

@Serializable
data class PartialChannelDataUpdate(
    val level: Float? = null,
    val pan: Float? = null,
    val reverb: Float? = null,
    val mute: Boolean? = null,
    val eqHighLevel: Float? = null,
    val eqHighFreq: Float? = null,
    val eqMidLevel: Float? = null,
    val eqMidFreq: Float? = null,
    val eqMidQ: Float? = null,
    val eqLowLevel: Float? = null,
    val eqLowFreq: Float? = null,
)

@Serializable
data class PartialGlobalDataUpdate(
    val masterLevel: Float? = null,
    val monitorLevel: Float? = null,
)

@Serializable
data class MixerPresets(
    val channels: List<PartialChannelDataUpdate> = emptyList(),
    val globals: PartialGlobalDataUpdate = PartialGlobalDataUpdate(),
)

@Serializable
data class MixerPreferences(
    val default: MixerPresets,
    val scenes: Map<String, MixerPresets>,
)
