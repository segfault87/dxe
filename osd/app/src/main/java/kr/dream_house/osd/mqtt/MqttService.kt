package kr.dream_house.osd.mqtt

import android.annotation.SuppressLint
import android.app.admin.DevicePolicyManager
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.lifecycle.LifecycleService
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kr.dream_house.osd.DeviceAdminReceiver
import kr.dream_house.osd.MqttConfig
import kr.dream_house.osd.R
import kr.dream_house.osd.mqtt.topics.SetScreenState
import kr.dream_house.osd.mqttConfigFlow
import org.eclipse.paho.client.mqttv3.MqttConnectOptions
import java.util.UUID
import kotlin.reflect.full.companionObjectInstance

class MqttService : LifecycleService() {

    companion object {
        const val TAG = "MqttService"

        private const val SERVICE_ID = 100

        private var IS_RUNNING = false

        fun startIfNotRunning(context: Context) {
            if (IS_RUNNING)
                return

            val intent = Intent(context, MqttService::class.java)
            context.startService(intent)

            IS_RUNNING = true
        }
    }

    class BootIntentReceiver : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
                val serviceIntent = Intent(context, MqttService::class.java)
                context.startForegroundService(serviceIntent)
            }
        }
    }

    inner class LocalBinder : Binder() {
        inline fun <reified T> subscribe(callback: TopicSubscriber<T>, qos: Int = 1) {
            this@MqttService.subscribe(callback, qos)
        }

        inline fun <reified T> unsubscribe(callback: TopicSubscriber<T>): Boolean {
             return this@MqttService.unsubscribe(callback)
        }

        inline fun <reified T> publish(topic: String, payload: T, qos: Int, retained: Boolean): Boolean {
            val serialized = Json.encodeToString(payload)

            return this@MqttService.publish(topic, serialized, qos, retained)
        }
    }

    val callbacks = mutableMapOf<String, SubscriberBundle<*>>()
    val subscriptions = mutableMapOf<String, Int>()
    val events = Channel<MqttEvent>()

    private lateinit var powerManager: PowerManager
    private lateinit var devicePolicyManager: DevicePolicyManager
    private lateinit var adminComponent: ComponentName

    private var mqttClientJob: Job? = null

    inline fun <reified T> subscribe(callback: TopicSubscriber<T>, qos: Int = 1) {
        val companion = T::class.companionObjectInstance as? TopicSpec ?: throw IllegalStateException("Type ${T::class} is not a topic")
        val topic = companion.topicName

        if (!callbacks.containsKey(topic)) {
            val bundle = SubscriberBundle(topic, object: Deserializer<T> {
                override fun deserialize(value: ByteArray): T {
                    return Json.decodeFromString(value.decodeToString())
                }
            })
            bundle.addCallback(callback)
            callbacks[topic] = bundle
            subscriptions[topic] = qos
            events.trySend(
                MqttEvent.Subscribe(
                    topic = topic,
                    qos = qos,
                )
            )
        } else {
            @Suppress("UNCHECKED_CAST") val bundle: SubscriberBundle<T> = callbacks[topic] as SubscriberBundle<T>
            bundle.addCallback(callback)
        }
    }

    inline fun <reified T> unsubscribe(callback: TopicSubscriber<T>): Boolean {
        var removed = false

        val keysToRemove = mutableListOf<String>()

        for ((key, bundle) in callbacks) {
            @Suppress("UNCHECKED_CAST") val bundle = bundle as SubscriberBundle<T>

            if (bundle.removeCallback(callback)) {
                removed = true
            }
            if (bundle.isEmpty()) {
                keysToRemove.add(key)
            }
        }

        for (key in keysToRemove) {
            subscriptions.remove(key)
            callbacks.remove(key)
            events.trySend(MqttEvent.Unsubscribe(
                topic = key
            ))
        }

        return removed
    }

    fun publish(topic: String, serializedPayload: String, qos: Int, retained: Boolean): Boolean {
        if (mqttClientJob == null) {
            return false
        }

        events.trySend(MqttEvent.Publish(
            topic = topic,
            payload = serializedPayload,
            qos = qos,
            retained = retained,
        ))

        return true
    }

    override fun onCreate() {
        super.onCreate()

        powerManager = getSystemService(PowerManager::class.java)
        devicePolicyManager = getSystemService(DevicePolicyManager::class.java)
        adminComponent = ComponentName(this, DeviceAdminReceiver::class.java)
    }

    override fun onBind(intent: Intent): IBinder {
        super.onBind(intent)
        return LocalBinder()
    }

    override fun onUnbind(intent: Intent?): Boolean {
        super.onUnbind(intent)

        return false
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)

        Log.d(TAG, "onStartCommand: $intent $flags $startId")

        startForeground()

        lifecycleScope.launch(Dispatchers.IO) {
            val prefs = mqttConfigFlow(this@MqttService)

            prefs.collect { config ->
                if (config == null) {
                    Log.w(TAG, "MQTT URL is not set. Waiting for configuration...")
                } else {
                    mqttClientJob?.cancel()
                    mqttClientJob = launch {
                        while (true) {
                            mqttClientLoop(config)
                            delay(1000)
                        }
                    }
                }
            }
        }

        return START_STICKY_COMPATIBILITY
    }

    private fun startForeground() {
        val notification = NotificationCompat.Builder(this, "FOREGROUND_SERVICE_NOTIFIER")
            .setContentTitle("DXE OSD is running...")
            .setSmallIcon(R.drawable.ic_logo)
            .build()

        Log.d(TAG, "Starting foreground service")

        ServiceCompat.startForeground(
            this,
            SERVICE_ID,
            notification,
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_REMOTE_MESSAGING
            } else {
                0
            }
        )
    }

    private fun subscribeServiceTopics() {
        subscribe(object: TopicSubscriber<SetScreenState> {
            override fun onPayload(topic: String, payload: SetScreenState) {
                if (payload.isActive) {
                    val wakeLock = powerManager.newWakeLock(
                        PowerManager.SCREEN_BRIGHT_WAKE_LOCK or PowerManager.ACQUIRE_CAUSES_WAKEUP or PowerManager.ON_AFTER_RELEASE,
                        "Dxe:SetScreenState"
                    )
                    @SuppressLint("WakelockTimeout")
                    wakeLock.acquire()
                } else {
                    if (devicePolicyManager.isAdminActive(adminComponent)) {
                        devicePolicyManager.lockNow()
                    } else {
                        Log.w(TAG, "Not turning screen off as device admin is not active.")
                    }
                }
            }
        })
    }

    private suspend fun mqttClientLoop(config: MqttConfig) {
        val uri = "tcp://${config.host}:${config.port}"
        val client = AsyncMqttClient(lifecycleScope, this, uri, "OSD_${UUID.randomUUID()}")

        Log.d(TAG, "Connecting to MQTT...")

        val connectOptions = MqttConnectOptions()
        if (config.username != null && config.password != null) {
            connectOptions.userName = config.username
            connectOptions.password = config.password.toCharArray()
        }

        val disconnectionTrigger = try {
            client.connect(connectOptions)
        } catch (e: Throwable) {
            Log.e(TAG, "Could not connect to MQTT server", e)
            return
        }

        Log.d(TAG, "Connected to MQTT endpoint ($uri)")

        for ((subscription, qos) in subscriptions) {
            client.subscribe(subscription, qos)
        }

        lifecycleScope.launch {
            events.receiveAsFlow().collect { event ->
                when (event) {
                    is MqttEvent.Subscribe -> {
                        client.subscribe(event.topic, event.qos)
                    }
                    is MqttEvent.Unsubscribe -> {
                        client.unsubscribe(event.topic)
                    }
                    is MqttEvent.Publish -> {
                        client.publish(event.topic, event.payload, event.qos, event.retained)
                    }
                }
            }
        }

        val flow = client.getMessageFlow()
        lifecycleScope.launch {
            flow.collect { (topic, message) ->
                val bundle = callbacks[topic]

                if (bundle != null) {
                    bundle.invoke(message.payload)
                } else {
                    Log.w(TAG, "No handler found for topic $topic")
                }
            }
        }

        subscribeServiceTopics()

        disconnectionTrigger.await()
    }
}