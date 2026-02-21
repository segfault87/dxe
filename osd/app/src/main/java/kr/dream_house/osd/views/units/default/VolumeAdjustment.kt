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
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.ui.theme.LabelBackground
import kr.dream_house.osd.utils.fractionalOffset
import kr.dream_house.osd.views.IndicatorArrow
import kr.dream_house.osd.views.InstructionPage

@Composable
private fun VolumeAdjustmentStep1() {
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
            text = "믹서 설정 화면에서 각 마이크 채널과 마스터 볼륨을 설정해 주세요.\n너무 높으면 하울링이 발생할 수 있습니다.")
    }
}

@Composable
private fun VolumeAdjustmentStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_volume_step2),
            contentScale = ContentScale.FillWidth,
            contentDescription = "모니터링 스피커"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.27f, 0.15f).rotate(-90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.27f, 0.15f, xOffset = 50.dp, yOffset = 50.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "볼륨 조절")

        Text(
            modifier = Modifier.fractionalOffset(0.95f, 0.75f, xOffset = (-560).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "뒷편에 있는 모니터링 스피커의 볼륨을 조정해주세요.\n너무 높으면 하울링이 발생할 수 있습니다.")
    }
}

@Composable
fun VolumeAdjustment(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { VolumeAdjustmentStep1() },
            { VolumeAdjustmentStep2() },
        ),
        onClose = onClose,
    )
}
