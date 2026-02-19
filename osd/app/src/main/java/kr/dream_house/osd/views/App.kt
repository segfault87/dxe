package kr.dream_house.osd.views

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonNull
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.Navigation.MainScreen.Companion.getSubroute
import kr.dream_house.osd.entities.AlertData
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.MixerPreferences
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.midi.ChannelData
import kr.dream_house.osd.midi.GlobalData
import kr.dream_house.osd.midi.LocalMixerController
import kr.dream_house.osd.midi.MixerState
import kr.dream_house.osd.midi.updateFrom
import kr.dream_house.osd.mqtt.MqttService
import kr.dream_house.osd.mqtt.TopicSubscriber
import kr.dream_house.osd.mqtt.topics.Alert
import kr.dream_house.osd.mqtt.topics.CurrentSession
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult
import kr.dream_house.osd.mqtt.topics.DoorbellRequest
import kr.dream_house.osd.mqtt.topics.ParkingStates
import kr.dream_house.osd.mqtt.topics.SetMixerPreferences
import kr.dream_house.osd.mqtt.topics.SetMixerStates
import kr.dream_house.osd.mqttConfigFlow
import kr.dream_house.osd.navigate
import kotlin.collections.component1
import kotlin.collections.component2
import kotlin.collections.iterator

const val SUBPAGE_TIMEOUT_MILLISECONDS: Long = 1000 * 60 * 5

@OptIn(FlowPreview::class)
@Composable
fun App(
    modifier: Modifier = Modifier,
    navController: NavHostController,
    startDestination: String? = null
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()
    val mqttConfig by mqttConfigFlow(context).collectAsState(null)
    val mixerController = LocalMixerController.current

    val mixerState by mixerController?.state?.collectAsState() ?: remember { mutableStateOf(MixerState())  }
    var activeBooking by remember { mutableStateOf<Booking?>(null) }
    var mixerPreferences by remember { mutableStateOf<MixerPreferences?>(null) }
    var parkingStates by remember { mutableStateOf<List<ParkingState>>(emptyList()) }
    var currentAlert by remember { mutableStateOf<AlertData?>(null) }
    var sendOpenDoorRequest by remember { mutableStateOf<suspend () -> DoorLockOpenResult?>({ null }) }
    var publishMixerState by remember { mutableStateOf<suspend (MixerState) -> Unit>({}) }
    var updateMixerPreferences by remember { mutableStateOf<(MixerPreferences) -> Unit>({}) }
    var doorbellRequest by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            // Quick hack for avoid getting initial state
            delay(500)
            if (mqttConfig == null) {
                navController.navigate(navigation = Navigation.Config)
            }
        }
    }

    LaunchedEffect(Unit) {
        snapshotFlow { mixerState }
            .debounce(5000L)
            .distinctUntilChanged()
            .collect {
                publishMixerState(it)
            }
    }

    var mqttServiceBinder by remember { mutableStateOf<MqttService.LocalBinder?>(null) }
    val connection = remember {
        object: ServiceConnection {
            override fun onServiceConnected(
                name: ComponentName?,
                service: IBinder?
            ) {
                mqttServiceBinder = service as? MqttService.LocalBinder?
            }

            override fun onServiceDisconnected(name: ComponentName?) {
                mqttServiceBinder = null
            }
        }
    }

    DisposableEffect(Unit) {
        val intent = Intent(context, MqttService::class.java)
        context.bindService(intent, connection, Context.BIND_AUTO_CREATE)

        onDispose {
            context.unbindService(connection)
        }
    }

    DisposableEffect(mqttServiceBinder) {
        mqttServiceBinder?.let { service ->
            val onCurrentSession = object: TopicSubscriber<CurrentSession> {
                override fun onPayload(
                    topic: String,
                    payload: CurrentSession
                ) {
                    activeBooking = payload.booking
                }

            }
            val onParkingStates = object: TopicSubscriber<ParkingStates> {
                override fun onPayload(
                    topic: String,
                    payload: ParkingStates
                ) {
                    parkingStates = payload.states
                }
            }
            val onAlert = object: TopicSubscriber<Alert> {
                override fun onPayload(topic: String, payload: Alert) {
                    currentAlert = payload.alert
                }
            }
            val doorLockOpenResultChannel = Channel<DoorLockOpenResult>(1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
            val onDoorLockOpenResult = object: TopicSubscriber<DoorLockOpenResult> {
                override fun onPayload(
                    topic: String,
                    payload: DoorLockOpenResult
                ) {
                    doorLockOpenResultChannel.trySend(payload)
                }
            }
            val onSetMixerStates = object: TopicSubscriber<SetMixerStates> {
                override fun onPayload(topic: String, payload: SetMixerStates) {
                    mixerController?.let { controller ->
                        if (payload.overwrite) {
                            val initialChannelStates = mutableListOf<ChannelData>()
                            for (i in 0 until controller.channels.size) {
                                initialChannelStates.add(payload.channels[i.toString()]?.let {
                                    ChannelData().updateFrom(it)
                                } ?: ChannelData())
                            }
                            val initialGlobalStates = GlobalData().updateFrom(payload.globals)
                            coroutineScope.launch {
                                controller.updateInitialChannelStates(initialChannelStates, initialGlobalStates)
                            }
                        } else {
                            for ((channel, data) in payload.channels) {
                                controller.updateValues(channel.toInt(), data)
                            }
                            controller.updateValues(payload.globals)
                        }
                    }

                }
            }
            val onSetMixerPreferences = object: TopicSubscriber<SetMixerPreferences> {
                override fun onPayload(topic: String, payload: SetMixerPreferences) {
                    mixerController?.let { controller ->
                        val prefs = payload.prefs
                        mixerPreferences = prefs
                        val default = prefs.default
                        val initialChannelStates = mutableListOf<ChannelData>()
                        for (i in 0 until controller.channels.size) {
                            initialChannelStates.add(default.channels.getOrNull(i)?.let {
                                ChannelData().updateFrom(it)
                            } ?: ChannelData())
                        }
                        val initialGlobalStates = GlobalData().updateFrom(default.globals)
                        coroutineScope.launch {
                            controller.updateInitialChannelStates(initialChannelStates, initialGlobalStates)
                        }
                    }
                }
            }
            val onDoorbellRequest = object: TopicSubscriber<DoorbellRequest> {
                override fun onPayload(topic: String, payload: DoorbellRequest) {
                    if (payload.unitId == null || payload.unitId == BuildConfig.UNIT_ID) {
                        doorbellRequest = true
                    }
                }
            }

            updateMixerPreferences = { prefs ->
                activeBooking?.customerId?.let { customerId ->
                    service.publish(SetMixerPreferences.Update.TOPIC_NAME, SetMixerPreferences.Update(
                        customerId = customerId,
                        prefs = prefs,
                    ), 1, false)
                }

                mixerPreferences = prefs
            }

            publishMixerState = {
                service.publish(SetMixerStates.syncTopicName, it, 1, false)
            }

            sendOpenDoorRequest = {
                val timeout = coroutineScope.launch {
                    delay(5000)
                    doorLockOpenResultChannel.trySend(DoorLockOpenResult(false, "통신에 실패했습니다."))
                }
                try {
                    if (!service.publish(DoorLockOpenResult.setTopicName, JsonNull, 1, false)) {
                        null
                    } else {
                        doorLockOpenResultChannel.receive()
                    }
                } finally {
                    timeout.cancel()
                }
            }

            service.subscribe(onDoorLockOpenResult)
            service.subscribe(onCurrentSession)
            service.subscribe(onAlert)
            service.subscribe(onParkingStates)
            service.subscribe(onSetMixerPreferences)
            service.subscribe(onSetMixerStates)
            service.subscribe(onDoorbellRequest)

            return@DisposableEffect onDispose {
                service.unsubscribe(onDoorbellRequest)
                service.unsubscribe(onSetMixerStates)
                service.unsubscribe(onSetMixerPreferences)
                service.unsubscribe(onParkingStates)
                service.unsubscribe(onAlert)
                service.unsubscribe(onCurrentSession)
                service.unsubscribe(onDoorLockOpenResult)
                sendOpenDoorRequest = { null }
                publishMixerState = {}
                updateMixerPreferences = {}
            }
        }

        return@DisposableEffect onDispose {}
    }

    Scaffold(topBar = {
        TopBar(modifier = Modifier.background(Color.White), activeBooking = activeBooking)
    }) { paddingValues ->
        NavHost(modifier = modifier.padding(paddingValues), navController = navController, startDestination = startDestination ?: Navigation.default().qualifiedRoute()) {
            composable(Navigation.MainScreen.routePattern()) { backStackEntry ->
                val subroute = backStackEntry.arguments.getSubroute() ?: Navigation.MainScreen.default().route()

                MainScreen(
                    initialSubroute = subroute,
                    sendOpenDoorRequest = sendOpenDoorRequest,
                    activeBooking = activeBooking,
                    parkingStates = parkingStates,
                    mixerPreferences = mixerPreferences,
                    onUpdateMixerPreferences = updateMixerPreferences,
                )
            }
            composable(Navigation.Config.qualifiedRoute()) {
                Config(onDismiss = {
                    navController.navigate(navigation = Navigation.default())
                })
            }
        }
    }

    currentAlert?.let {
        AlertPopup(
            it.title,
            it.contents,
            if (it.closeable) {
                { currentAlert = null }
            } else {
                null
            },
            it.severity
        )
    }

    if (doorbellRequest) {
        DoorbellPopup(
            sendOpenDoorRequest,
            {
                doorbellRequest = false
            }
        )
    }
}