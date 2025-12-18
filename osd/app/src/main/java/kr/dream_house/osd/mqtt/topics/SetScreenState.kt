package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class SetScreenState(val isActive: Boolean) {
    companion object : TopicSpec {
        override val topicName = "dxe/screen_state/${BuildConfig.UNIT_ID}"
    }
}