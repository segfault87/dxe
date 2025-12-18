package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.entities.AlertData
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class Alert(
    val alert: AlertData?
) {
    companion object : TopicSpec {
        override val topicName = "dxe/alert/${BuildConfig.UNIT_ID}"
    }
}