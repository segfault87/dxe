package kr.dream_house.osd.views.unit_default

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.ui.theme.LabelBackground
import kr.dream_house.osd.utils.fractionalOffset
import kr.dream_house.osd.views.IndicatorArrow

@Composable
private fun CheckMixerPower() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_mixer_step1),
            contentScale = ContentScale.Crop,
            contentDescription = "믹서 전원을 확인해 주세요."
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.68f, 0.2f, xOffset = (-48).dp))
        Text(
            modifier = Modifier.fractionalOffset(0.65f, 0.25f, xOffset = (-190).dp, yOffset = 42.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "전원 스위치")

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "믹서 전원 스위치가 켜져 있는지 확인해 주세요.")
    }
}

@Composable
private fun CheckMixerLevel() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_mixer_step2),
            contentScale = ContentScale.Crop,
            contentDescription = "믹서 레벨을 확인해 주세요."
        )

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.8f, yOffset = (-20).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "페이더를 올려 주세요.\n무선 마이크는 3, 4번 채널, 건반은 11-12 채널, 전체 볼륨은 빨간색입니다.")
    }
}

@Composable
private fun CheckMicPower() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fractionalOffset(0.8f, 0.1f, xOffset = (-60).dp),
            painter = painterResource(R.drawable.img_mixer_step3),
            contentDescription = "마이크 전원"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.75f, 0.7f, xOffset = (-48).dp))
        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "무선 마이크의 전원 스위치가 켜져 있는지 확인해 주세요.")
    }
}

@Composable
private fun CheckMicReceiver() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_mixer_step4),
            contentScale = ContentScale.Crop,
            contentDescription = "무선 마이크 수신기 전원 상태를 확인해 주세요."
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.86f, 0.65f, xOffset = (-48).dp))
        Text(
            modifier = Modifier.fractionalOffset(0.86f, 0.65f, xOffset = (-200).dp, yOffset = 48.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "전원 스위치")

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "무선 마이크 수신기의 전원이 켜져 있는지 확인해 주세요.")
    }
}

@Composable
private fun CheckMicChannels() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_mixer_step5),
            contentScale = ContentScale.Crop,
            contentDescription = "무선 마이크 채널을 확인해 주세요."
        )

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.8f, xOffset = (-175).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "채널이 일치하는지 확인해 주세요.\n양 옆 화살표 버튼으로 조정할 수 있습니다.")
    }
}

@Composable
private fun CheckMute() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_mixer_step6),
            contentScale = ContentScale.Crop,
            contentDescription = "채널 뮤트 상태 확인"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.25f, 0.42f).rotate(-90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.25f, 0.42f, xOffset = 48.dp, yOffset = 48.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "누른 뒤")

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.6f, 0.42f).rotate(-90.0f), offsetMillis = 250)
        Text(
            modifier = Modifier.fractionalOffset(0.6f, 0.42f, xOffset = 48.dp, yOffset = 48.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "켜져 있으면 끄기")

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "채널이 뮤트 상태인지 확인해 주세요.\n확인 후 MUTE 버튼을 한번 더 눌러서 꺼주세요.")
    }
}

@Composable
fun MixerSetup(onClose: () -> Unit) {
    var page by remember { mutableStateOf(0) }

    if (page == 6) {
        TroubleshootingContact("그럼에도 소리가 계속 나오지 않는다면\n위 연락처로 문의 바랍니다.")
    } else {
        Column(modifier = Modifier.fillMaxSize()) {
            Box(modifier = Modifier.weight(1.0f)) {
                when (page) {
                    0 -> CheckMixerPower()
                    1 -> CheckMixerLevel()
                    2 -> CheckMicPower()
                    3 -> CheckMicReceiver()
                    4 -> CheckMicChannels()
                    5 -> CheckMute()
                }
            }
            Column {
                Text(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                    text = "소리가 정상적으로 나나요?",
                    textAlign = TextAlign.Center,
                    style = MaterialTheme.typography.headlineMedium
                )
                Row(modifier = Modifier.fillMaxWidth().height(64.dp)) {
                    TextButton(
                        modifier = Modifier.weight(1.0f).fillMaxHeight(),
                        onClick = onClose,
                        colors = ButtonDefaults.textButtonColors(contentColor = Color.Black)
                    ) {
                        Image(
                            painter = painterResource(R.drawable.ic_troubleshooting_yes),
                            contentDescription = "Yes"
                        )
                        Text(
                            modifier = Modifier.padding(start = 16.dp),
                            style = MaterialTheme.typography.headlineSmall,
                            text = "네"
                        )
                    }
                    VerticalDivider()
                    TextButton(
                        modifier = Modifier.weight(1.0f).fillMaxHeight(),
                        onClick = { page += 1 },
                        colors = ButtonDefaults.textButtonColors(contentColor = Color.Black)
                    ) {
                        Image(
                            painter = painterResource(R.drawable.ic_troubleshooting_no),
                            contentDescription = "No"
                        )
                        Text(
                            modifier = Modifier.padding(start = 16.dp),
                            style = MaterialTheme.typography.headlineSmall,
                            text = "아니오"
                        )
                    }
                }
            }
        }
    }
}
