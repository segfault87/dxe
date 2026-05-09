package kr.dream_house.osd.midi

import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import kr.dream_house.osd.MixerChannelId

@Serializable
data class MixerChannelConfig(
    val id: MixerChannelId,
    val name: String,
    val index: Int,
    val stereo: Boolean = false,
    val capabilityReverb: Boolean = true,
    val capabilityEq: Boolean = true,
    val capabilityBalance: Boolean = true,
    val capabilityMute: Boolean = true,
)

@Serializable
data class MixerConfigurations(
    val channels: List<MixerChannelConfig>
) {

    @Transient
    val channelsById = channels.associateBy { it.id }

    @Transient
    val channelsByIndex = channels.associateBy { it.index }

}