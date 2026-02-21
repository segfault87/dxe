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
private fun PersonalMonitorStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_personal_monitor_step1),
            contentScale = ContentScale.FillWidth,
            contentDescription = ""
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.7f, 0.3f))

        Text(
            modifier = Modifier.fractionalOffset(0.7f, 0.3f, xOffset = (-384).dp, yOffset = 64.dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "모니터링 이어폰/헤드폰/무선 송신기를\n여기에 연결해주세요.")
    }
}

@Composable
private fun PersonalMonitorStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_personal_monitor_step2),
            contentScale = ContentScale.FillWidth,
            contentDescription = "믹서 설정"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.95f, 0.7f, xOffset = (-580).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "믹서 설정 화면에서 개인 모니터 음량을 설정하실 수 있습니다.")
    }
}

@Composable
fun PersonalMonitor(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { PersonalMonitorStep1() },
            { PersonalMonitorStep2() },
        ),
        onClose = onClose,
    )
}
