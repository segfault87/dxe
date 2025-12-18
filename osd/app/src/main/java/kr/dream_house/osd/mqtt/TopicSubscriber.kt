package kr.dream_house.osd.mqtt

interface TopicSubscriber<T> {
    fun onPayload(topic: String, payload: T)
}

interface Deserializer<T> {
    fun deserialize(value: ByteArray): T
}

class SubscriberBundle<T>(val topic: String, val deserializer: Deserializer<T>) {

    val callbacks = mutableListOf<TopicSubscriber<T>>()

    fun addCallback(callback: TopicSubscriber<T>): Boolean {
        if (!callbacks.contains(callback)) {
            callbacks.add(callback)
            return true
        } else {
            return false
        }
    }

    fun removeCallback(callback: TopicSubscriber<T>): Boolean {
        return callbacks.remove(callback)
    }

    fun isEmpty(): Boolean = callbacks.isEmpty()

    fun invoke(data: ByteArray) {
        if (callbacks.isEmpty()) {
            return
        }

        val deserialized = try {
            deserializer.deserialize(data)
        } catch (e: Exception) {
            e.printStackTrace()
            return
        }
        for (callback in callbacks) {
            callback.onPayload(topic, deserialized)
        }
    }
}