package kr.dream_house.osd.mqtt

import android.content.Context
import android.util.Log
import info.mqtt.android.service.MqttAndroidClient
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import org.eclipse.paho.client.mqttv3.DisconnectedBufferOptions
import org.eclipse.paho.client.mqttv3.IMqttActionListener
import org.eclipse.paho.client.mqttv3.IMqttDeliveryToken
import org.eclipse.paho.client.mqttv3.IMqttToken
import org.eclipse.paho.client.mqttv3.MqttCallback
import org.eclipse.paho.client.mqttv3.MqttCallbackExtended
import org.eclipse.paho.client.mqttv3.MqttConnectOptions
import org.eclipse.paho.client.mqttv3.MqttMessage
import kotlin.coroutines.resumeWithException
import kotlin.coroutines.suspendCoroutine

class AsyncMqttClient(context: Context, uri: String, clientId: String) {

    companion object {
        private const val TAG = "AsyncMqttClient"
    }

    val client = MqttAndroidClient(context, uri, clientId).also {
        it.setTraceEnabled(true)
    }

    val tokens = mutableMapOf<IMqttToken, CompletableDeferred<Unit>>()

    fun getMessageFlow(): Flow<Pair<String, MqttMessage>> = callbackFlow {
        val callback = object : MqttCallback {
            override fun connectionLost(cause: Throwable?) {}

            override fun messageArrived(
                topic: String,
                message: MqttMessage
            ) {
                try {
                    trySend(Pair(topic, message))
                } catch(e: Throwable) {
                    Log.e(TAG, "Couldn't send mqtt message through flow: $e")
                }
            }

            override fun deliveryComplete(token: IMqttDeliveryToken) {}
        }

        Log.d(TAG, "Adding callback $callback")
        client.addCallback(callback)

        awaitClose {
            Log.d(TAG, "Removing callback $callback")
            client.removeCallback(callback)
        }
    }

    suspend fun connect(options: MqttConnectOptions) = suspendCoroutine { continuation ->
        val disconnectionTrigger = CompletableDeferred<Unit>()

        client.setCallback(object: MqttCallbackExtended {
            override fun connectComplete(reconnect: Boolean, serverURI: String?) {}

            override fun connectionLost(cause: Throwable) {
                disconnectionTrigger.complete(Unit)
            }

            override fun messageArrived(
                topic: String,
                message: MqttMessage
            ) {}

            override fun deliveryComplete(token: IMqttDeliveryToken) {
                tokens.remove(token)?.complete(Unit)
            }

        })

        client.connect(options, null, object : IMqttActionListener {
            override fun onSuccess(asyncActionToken: IMqttToken) {
                client.setBufferOpts(DisconnectedBufferOptions().apply {
                    isBufferEnabled = true
                    bufferSize = 100
                    isPersistBuffer = false
                    isDeleteOldestMessages = false
                })
                continuation.resumeWith(Result.success(disconnectionTrigger))
            }

            override fun onFailure(
                asyncActionToken: IMqttToken?,
                exception: Throwable
            ) {
                continuation.resumeWithException(exception)
            }

        })
    }

    fun subscribe(topic: String, qos: Int) {
        client.subscribe(topic, qos)
    }

    fun unsubscribe(topic: String) {
        client.unsubscribe(topic)
    }

    suspend fun publish(topic: String, payload: String, qos: Int, retained: Boolean) {
        val token = client.publish(topic, payload.toByteArray(), qos, retained)

        val signal = CompletableDeferred<Unit>()
        tokens[token] = signal
        signal.await()
    }

}