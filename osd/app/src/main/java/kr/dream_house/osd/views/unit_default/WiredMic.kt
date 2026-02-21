package kr.dream_house.osd.views.unit_default

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
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.utils.fractionalOffset
import kr.dream_house.osd.views.IndicatorArrow
import kr.dream_house.osd.views.InstructionPage

@Composable
private fun WiredMicStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_wired_mic_step1),
            contentScale = ContentScale.FillWidth,
            contentDescription = ""
        )

        Text(
            modifier = Modifier.fractionalOffset(0.15f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "유선 마이크는 믹서 아래 서랍 두번째 칸에 있습니다.")
    }
}

@Composable
private fun WiredMicStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_wired_mic_step2),
            contentScale = ContentScale.FillWidth,
            contentDescription = "믹서 설정"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.31f, 0.25f).rotate(-90.0f))

        Text(
            modifier = Modifier.fractionalOffset(0.31f, 0.25f, xOffset = 64.dp, yOffset = 64.dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "채널 1, 2의 XLR 단자에 마이크를 연결해 주세요.\n케이블은 입구 쪽에 있습니다.")
    }
}

@Composable
private fun WiredMicStep3() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_wired_mic_step3),
            contentScale = ContentScale.FillWidth,
            contentDescription = "믹서 설정"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.3f, 0.65f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "믹서 설정 화면에서 채널 1, 2번의 음량을 조정해 주세요.")
    }
}

@Composable
fun WiredMic(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { WiredMicStep1() },
            { WiredMicStep2() },
            { WiredMicStep3() },
        ),
        onClose = onClose,
    )
}
