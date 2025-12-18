package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class CurrentSession(val booking: Booking?) {
    companion object : TopicSpec {
        override val topicName = "dxe/current_session/${BuildConfig.UNIT_ID}"
    }
}