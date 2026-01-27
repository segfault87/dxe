package kr.dream_house.osd.views

import android.media.MediaPlayer
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kr.dream_house.osd.R
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult

@Composable
fun DoorbellPopup(sendOpenDoorRequest: (suspend () -> DoorLockOpenResult?), onDismiss: (() -> Unit)) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()

    var timerTask by remember { mutableStateOf<Job?>(null) }

    DisposableEffect(Unit) {
        timerTask = coroutineScope.launch {
            delay(30000)
            onDismiss()
        }

        onDispose {
            timerTask?.cancel()
        }
    }

    var isInProgress by remember { mutableStateOf(false) }

    DisposableEffect(Unit) {
        val mediaPlayer = MediaPlayer.create(context, R.raw.doorbell).apply {
            isLooping = true
            start()
        }

        onDispose {
            mediaPlayer.stop()
            mediaPlayer.release()
        }
    }

    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            dismissOnBackPress = false,
            dismissOnClickOutside = false,
        )
    ) {
        Surface(
            modifier = Modifier.width(800.dp).shadow(16.dp, shape = MaterialTheme.shapes.medium),
            shape = RoundedCornerShape(16.dp),
            color = Color.White
        ) {
            Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                Text(modifier = Modifier.fillMaxWidth(), text = "입구에서 초인종이 눌렸습니다.", textAlign = TextAlign.Center, style = MaterialTheme.typography.headlineLarge)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                Text(modifier = Modifier.fillMaxWidth().padding(top = 32.dp, bottom = 32.dp), text = "문을 열까요?", style = MaterialTheme.typography.titleLarge)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    TextButton(
                        modifier = Modifier.weight(1.0f),
                        enabled = !isInProgress,
                        onClick = {
                            coroutineScope.launch {
                                isInProgress = true
                                val result = sendOpenDoorRequest()
                                if (result?.success == true) {
                                    onDismiss()
                                    Toast.makeText(context, "문이 열렸습니다.", Toast.LENGTH_SHORT).show()
                                } else if (result?.error != null) {
                                    Toast.makeText(context, "문 열기에 실패했습니다: ${result.error}", Toast.LENGTH_SHORT).show()
                                } else {
                                    Toast.makeText(context, "문 열기에 실패했습니다.", Toast.LENGTH_SHORT).show()
                                }
                                isInProgress = false
                            }
                        },
                        colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.secondary)
                    ) {
                        Text("네", style = MaterialTheme.typography.headlineSmall)
                    }
                    TextButton(
                        modifier = Modifier.weight(1.0f),
                        onClick = onDismiss, colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.tertiary)
                    ) {
                        Text("아니오", style = MaterialTheme.typography.headlineSmall)
                    }
                }
            }
        }
    }
}
