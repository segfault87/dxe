package kr.dream_house.osd.views

import android.widget.Toast
import androidx.compose.animation.animateColor
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult
import kr.dream_house.osd.utils.elapsedTimeFlow
import kr.dream_house.osd.utils.format
import kotlin.time.Duration

private const val PARKING_NOTIFY_THRESHOLD_MINS = 110

@Composable
fun ParkingEntry(
    parkingState: ParkingState
) {
    val elapsedTime by parkingState.elapsedTimeFlow().collectAsState(Duration.ZERO)

    val color = if (elapsedTime.inWholeMinutes > PARKING_NOTIFY_THRESHOLD_MINS) {
        val transition = rememberInfiniteTransition()
        val animatedColor by transition.animateColor(
            initialValue = Color.Black,
            targetValue = Color.Red,
            animationSpec = infiniteRepeatable(
                animation = tween(durationMillis = 250, easing = LinearEasing),
                repeatMode = RepeatMode.Reverse,
            )
        )

        animatedColor
    } else {
        Color.Black
    }

    val text = buildAnnotatedString {
        append("• ")
        withStyle(style = SpanStyle(color = color)) {
            append("${parkingState.userName} (${parkingState.licensePlateNumber}): ")
            withStyle(style = SpanStyle(fontWeight = FontWeight.Bold)) {
                append(elapsedTime.format())
            }
        }
        parkingState.fuzzy?.let {
            append(" (${it}로 인식)")
        }
    }

    Text(
        modifier = Modifier.padding(start = 32.dp, top = 16.dp, end = 16.dp),
        style = MaterialTheme.typography.bodyMedium,
        text = text)
}

@Composable
fun ColumnScope.RealTimeInformation(
    sendOpenDoorRequest: suspend () -> DoorLockOpenResult?,
    activeBooking: Booking?,
    parkingStates: List<ParkingState>
) {
    val coroutineScope = rememberCoroutineScope()
    val context = LocalContext.current

    var doorUnlockInProgress by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier.weight(1.0f).verticalScroll(rememberScrollState())
    ) {
        Text(modifier = Modifier.padding(16.dp), text = "입차 정보", style = MaterialTheme.typography.titleLarge)
        if (parkingStates.isNotEmpty()) {
            for (parkingState in parkingStates) {
                ParkingEntry(parkingState)
            }
        } else {
            Text(
                modifier = Modifier.padding(start = 32.dp, top = 16.dp, end = 16.dp),
                style = MaterialTheme.typography.bodyMedium,
                text = "입차된 차량 없음")
        }
    }
    if (activeBooking != null) {
        ElevatedButton(
            modifier = Modifier.fillMaxWidth().padding(8.dp),
            enabled = !doorUnlockInProgress,
            colors = ButtonDefaults.elevatedButtonColors(
                contentColor = MaterialTheme.colorScheme.tertiary
            ),
            onClick = {
                coroutineScope.launch {
                    doorUnlockInProgress = true
                    val result = sendOpenDoorRequest()
                    val success = result?.success ?: false
                    val message = if (success) {
                        "문이 열렸습니다."
                    } else {
                        result?.error ?: "일시적인 오류가 발생했습니다."
                    }
                    Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
                    doorUnlockInProgress = false
                }
            }
        ) {
            Text(
                modifier = Modifier.padding(16.dp),
                text = "출입문 열기",
                style = MaterialTheme.typography.titleLarge
            )
        }
    }
}