package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.entities.PartialChannelDataUpdate
import kr.dream_house.osd.entities.PartialGlobalDataUpdate
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class SetMixerStates(
    val overwrite: Boolean,
    val channels: Map<String, PartialChannelDataUpdate> = emptyMap(),
    val globals: PartialGlobalDataUpdate = PartialGlobalDataUpdate(),
) {
    companion object : TopicSpec {
        override val topicName = "dxe/mixer_states/${BuildConfig.UNIT_ID}"

        val syncTopicName = "dxe/mixer_states/${BuildConfig.UNIT_ID}/sync"
    }
}