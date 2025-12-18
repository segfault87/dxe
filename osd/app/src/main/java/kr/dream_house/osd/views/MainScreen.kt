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
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonNull
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.entities.AlertData
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.mqtt.MqttService
import kr.dream_house.osd.mqtt.TopicSubscriber
import kr.dream_house.osd.mqtt.topics.Alert
import kr.dream_house.osd.mqtt.topics.CurrentSession
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult
import kr.dream_house.osd.mqtt.topics.ParkingStates
import kr.dream_house.osd.mqttConfigFlow
import kr.dream_house.osd.ui.theme.WhitishYellow
import kr.dream_house.osd.views.unit_default.UnitInformation

@Composable
fun MainScreen(navController: NavHostController) {
    val context = LocalContext.current
    val mqttConfig by mqttConfigFlow(context).collectAsState(null)
    val coroutineScope = rememberCoroutineScope()

    var activeBooking by remember { mutableStateOf<Booking?>(null) }
    var parkingStates by remember { mutableStateOf<List<ParkingState>>(emptyList()) }
    var currentAlert by remember { mutableStateOf<AlertData?>(null) }
    var sendOpenDoorRequest by remember { mutableStateOf<suspend () -> DoorLockOpenResult?>({ null }) }

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            // Quick hack for avoid getting initial state
            delay(500)
            if (mqttConfig == null) {
                navController.navigate(route = Navigation.Config)
            }
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
        mqttServiceBinder?.let {
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

            sendOpenDoorRequest = {
                if (!it.publish(DoorLockOpenResult.setTopicName, JsonNull, 1, false)) {
                    null
                } else {
                    doorLockOpenResultChannel.receive()
                }
            }

            it.subscribe(onDoorLockOpenResult)
            it.subscribe(onCurrentSession)
            it.subscribe(onAlert)
            it.subscribe(onParkingStates)

            return@DisposableEffect onDispose {
                it.unsubscribe(onParkingStates)
                it.unsubscribe(onAlert)
                it.unsubscribe(onCurrentSession)
                it.unsubscribe(onDoorLockOpenResult)
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
                    RealTimeInformation(
                        sendOpenDoorRequest = sendOpenDoorRequest,
                        activeBooking = activeBooking,
                        parkingStates = parkingStates
                    )
                }
            }
            Box(modifier = Modifier.weight(0.7f)) {
                UnitInformation()
            }
        }
    }

    currentAlert?.let {
        ModalPopup(
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
}