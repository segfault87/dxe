package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class ControlDevice(
    val lock: Boolean? = null,
    val navigation: String? = null,
    val finish: Boolean? = null,
    val triggerCrash: Boolean? = null,
    val omitScreenState: Boolean? = null,
) {
    companion object : TopicSpec {
        override val topicName = "dxe/control_device/${BuildConfig.UNIT_ID}"
    }
}