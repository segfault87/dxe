package kr.dream_house.osd.views.units.default

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.utils.fractionalOffset
import kr.dream_house.osd.views.IndicatorArrow
import kr.dream_house.osd.views.InstructionPage

@Composable
private fun NoMicrophoneStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_no_microphone),
            contentScale = ContentScale.FillWidth,
            contentDescription = "무선 마이크 수신기"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.1f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "무선 마이크 수신기의 전원이 켜져 있는지 확인해 주세요.")

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.8f, 0.57f, xOffset = (-48).dp))
    }
}

@Composable
private fun NoMicrophoneStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_volume_step1),
            contentScale = ContentScale.FillWidth,
            contentDescription = "믹서 설정"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.95f, 0.7f, xOffset = (-580).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "믹서 설정 화면에서 각 마이크 채널과 마스터 볼륨을 확인해 주세요.\n너무 높으면 하울링이 발생할 수 있습니다.")
    }
}

@Composable
fun NoMicrophone(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { NoMicrophoneStep1() },
            { NoMicrophoneStep2() },
        ),
        onClose = onClose,
    )
}
