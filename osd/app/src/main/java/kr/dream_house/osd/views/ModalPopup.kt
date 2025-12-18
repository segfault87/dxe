package kr.dream_house.osd.views

import android.media.MediaPlayer
import androidx.compose.animation.animateColor
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kr.dream_house.osd.AlertSeverity
import kr.dream_house.osd.R

@Composable
fun ModalPopup(title: String, contents: String, onDismiss: (() -> Unit)?, severity: AlertSeverity) {
    if (severity == AlertSeverity.INTRUSIVE) {
        val context = LocalContext.current

        DisposableEffect(Unit) {
            val mediaPlayer = MediaPlayer.create(context, R.raw.alarm).apply {
                isLooping = true
                start()
            }

            onDispose {
                mediaPlayer.stop()
                mediaPlayer.release()
            }
        }
    }

    if (severity != AlertSeverity.NORMAL) {
        val infiniteTransition = rememberInfiniteTransition()
        val scrimColor by infiniteTransition.animateColor(
            initialValue = Color(0x00ff0000),
            targetValue = Color.Red,
            animationSpec = infiniteRepeatable(
                animation = tween(durationMillis = 300, easing = FastOutSlowInEasing),
                repeatMode = RepeatMode.Reverse,
            )
        )
        Box(modifier = Modifier.fillMaxSize().background(scrimColor)) {}
    }

    Dialog(
        onDismissRequest = onDismiss ?: {},
        properties = DialogProperties(
            dismissOnBackPress = onDismiss != null,
            dismissOnClickOutside = onDismiss != null,
        )
    ) {
        Surface(
            modifier = Modifier.width(800.dp).shadow(16.dp, shape = MaterialTheme.shapes.medium),
            shape = RoundedCornerShape(16.dp),
            color = Color.White
        ) {
            Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                Text(modifier = Modifier.fillMaxWidth(), text = title, textAlign = TextAlign.Center, style = MaterialTheme.typography.headlineLarge)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                Text(modifier = Modifier.fillMaxWidth(), text = contents, style = MaterialTheme.typography.titleLarge)
                onDismiss?.let {
                    HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                    TextButton(modifier = Modifier.fillMaxWidth(), onClick = it, colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.tertiary)) {
                        Text("닫기", style = MaterialTheme.typography.headlineSmall)
                    }
                }
            }
        }
    }
}
