package kr.dream_house.osd.mqtt

sealed class MqttEvent {

    data class Subscribe(
        val topic: String,
        val qos: Int,
    ) : MqttEvent()

    data class Unsubscribe(
        val topic: String,
    ) : MqttEvent()

    data class Publish(
        val topic: String,
        val payload: String,
        val qos: Int,
        val retained: Boolean,
    ) : MqttEvent()

}