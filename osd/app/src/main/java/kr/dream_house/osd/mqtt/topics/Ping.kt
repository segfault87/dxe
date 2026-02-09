package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig

@Serializable
class Ping(val timestamp: Long) {
    companion object {
        const val OUTBOUND_TOPIC_NAME = "dxe/ping/${BuildConfig.UNIT_ID}"
    }
}