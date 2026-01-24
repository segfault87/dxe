package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.UnitId
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class DoorbellRequest(
    val unitId: UnitId? = null,
) {
    companion object : TopicSpec {
        override val topicName: String = "dxe/doorbell_request"
    }
}