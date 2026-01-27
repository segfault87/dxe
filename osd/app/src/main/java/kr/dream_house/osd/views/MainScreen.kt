package kr.dream_house.osd.views

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonNull
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.entities.AlertData
import kr.dream_house.osd.entities.Booking
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
import kr.dream_house.osd.mqtt.topics.SetMixerStates
import kr.dream_house.osd.mqttConfigFlow
import kr.dream_house.osd.ui.theme.WhitishYellow
import kr.dream_house.osd.views.unit_default.UnitInformation

const val SUBPAGE_TIMEOUT_MILLISECONDS: Long = 1000 * 60 * 5

enum class CurrentPage {
    Mixer,
    UnitInformation,
}

@OptIn(FlowPreview::class)
@Composable
fun MainScreen(navController: NavHostController) {
    val context = LocalContext.current
    val mqttConfig by mqttConfigFlow(context).collectAsState(null)
    val coroutineScope = rememberCoroutineScope()
    val mixerController = LocalMixerController.current
    var timerTask by remember { mutableStateOf<Job?>(null) }
    var currentPage by remember { mutableStateOf(CurrentPage.Mixer) }

    val mixerState by mixerController?.state?.collectAsState() ?: remember { mutableStateOf(MixerState())  }
    var activeBooking by remember { mutableStateOf<Booking?>(null) }
    var parkingStates by remember { mutableStateOf<List<ParkingState>>(emptyList()) }
    var currentAlert by remember { mutableStateOf<AlertData?>(null) }
    var sendOpenDoorRequest by remember { mutableStateOf<suspend () -> DoorLockOpenResult?>({ null }) }
    var publishMixerState by remember { mutableStateOf<suspend (MixerState) -> Unit>({}) }
    var doorbellRequest by remember { mutableStateOf(false) }

    LaunchedEffect(currentPage) {
        timerTask?.cancel()

        if (currentPage != CurrentPage.Mixer) {
            timerTask = null
        } else {
            timerTask = coroutineScope.launch {
                delay(SUBPAGE_TIMEOUT_MILLISECONDS)
                currentPage = CurrentPage.Mixer
            }
        }
    }

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            // Quick hack for avoid getting initial state
            delay(500)
            if (mqttConfig == null) {
                navController.navigate(route = Navigation.Config)
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
            val doorLockOpenResultChannel = Channel<DoorLockOpenResult>()
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
                            controller.updateInitialChannelStates(initialChannelStates, initialGlobalStates)
                        } else {
                            for ((channel, data) in payload.channels) {
                                controller.updateValues(channel.toInt(), data)
                            }
                            controller.updateValues(payload.globals)
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

            publishMixerState = {
                service.publish(SetMixerStates.syncTopicName, it, 1, false)
            }

            sendOpenDoorRequest = {
                if (!service.publish(DoorLockOpenResult.setTopicName, JsonNull, 1, false)) {
                    null
                } else {
                    doorLockOpenResultChannel.receive()
                }
            }

            service.subscribe(onDoorLockOpenResult)
            service.subscribe(onCurrentSession)
            service.subscribe(onAlert)
            service.subscribe(onParkingStates)
            service.subscribe(onSetMixerStates)
            service.subscribe(onDoorbellRequest)

            return@DisposableEffect onDispose {
                service.unsubscribe(onDoorbellRequest)
                service.unsubscribe(onSetMixerStates)
                service.unsubscribe(onParkingStates)
                service.unsubscribe(onAlert)
                service.unsubscribe(onCurrentSession)
                service.unsubscribe(onDoorLockOpenResult)
                sendOpenDoorRequest = { null }
                publishMixerState = {}
            }
        }

        return@DisposableEffect onDispose {}
    }

    Column(modifier = Modifier.fillMaxSize()) {
        TopBar(modifier = Modifier.background(Color.White), activeBooking = activeBooking)
        Row(modifier = Modifier.fillMaxWidth().weight(1.0f).background(WhitishYellow)) {
            Box(modifier = Modifier.weight(0.3f).padding(16.dp)) {
                Card(
                    colors = CardDefaults.cardColors(containerColor = Color(0xfffff1d7)),
                    elevation = CardDefaults.cardElevation(defaultElevation = 6.dp)
                ) {
                    Column {
                        Column(modifier = Modifier.fillMaxSize()) {
                            RealTimeInformation(
                                sendOpenDoorRequest = sendOpenDoorRequest,
                                activeBooking = activeBooking,
                                parkingStates = parkingStates
                            )

                            ElevatedButton(
                                modifier = Modifier.fillMaxWidth().padding(8.dp),
                                colors = ButtonDefaults.elevatedButtonColors(
                                    contentColor = MaterialTheme.colorScheme.tertiary
                                ),
                                onClick = {
                                    currentPage = CurrentPage.Mixer
                                },
                                enabled = currentPage != CurrentPage.Mixer,
                            ) {
                                Text(
                                    modifier = Modifier.padding(16.dp),
                                    text = "음량 설정",
                                    style = MaterialTheme.typography.titleLarge
                                )
                            }
                            ElevatedButton(
                                modifier = Modifier.fillMaxWidth().padding(8.dp),
                                colors = ButtonDefaults.elevatedButtonColors(
                                    contentColor = MaterialTheme.colorScheme.tertiary
                                ),
                                onClick = {
                                    currentPage = CurrentPage.UnitInformation
                                },
                                enabled = currentPage != CurrentPage.UnitInformation,
                            ) {
                                Text(
                                    modifier = Modifier.padding(16.dp),
                                    text = "도움말",
                                    style = MaterialTheme.typography.titleLarge
                                )
                            }
                        }
                    }
                }
            }
            Box(modifier = Modifier.weight(0.7f)) {
                when (currentPage) {
                    CurrentPage.Mixer -> MixerControls()
                    CurrentPage.UnitInformation -> UnitInformation()
                }
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