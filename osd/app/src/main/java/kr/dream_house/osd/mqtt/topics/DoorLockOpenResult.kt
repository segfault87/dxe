package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class DoorLockOpenResult(
    val success: Boolean,
    val error: String?,
) {
    companion object : TopicSpec {
        override val topicName = "dxe/doorlock/get"
        val setTopicName = "dxe/doorlock/set"
    }
}