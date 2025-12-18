package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BookingId
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
class StopSession(
    val bookingId: BookingId
) {
    companion object: TopicSpec {
        override val topicName = "dxe/stop_session/${BuildConfig.UNIT_ID}"
    }
}