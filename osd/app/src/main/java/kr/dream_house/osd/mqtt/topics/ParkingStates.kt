package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
class ParkingStates(val states: List<ParkingState>) {
    companion object : TopicSpec {
        override val topicName = "dxe/parking_states/${BuildConfig.UNIT_ID}"
    }
}