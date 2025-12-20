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

private const val MAX_PAGE_INDEX = 2

@Composable
private fun VolumeAdjustmentStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_volume_step3),
            contentScale = ContentScale.FillWidth,
            contentDescription = "모니터링 스피커"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.27f, 0.15f).rotate(-90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.27f, 0.15f, xOffset = 50.dp, yOffset = 50.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "볼륨 조절")

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.24f, 0.53f, yOffset = (-48).dp).rotate(180.0f), offsetMillis = 300)
        Text(
            modifier = Modifier.fractionalOffset(0.24f, 0.53f, xOffset = 50.dp, yOffset = (-100).dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "전원 스위치")

        Text(
            modifier = Modifier.fractionalOffset(0.95f, 0.75f, xOffset = (-560).dp)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "다른 악기와 앰프의 소리에 묻혀서 잘 들리지 않는 경우에는\n뒷편에 있는 모니터링 스피커를 추가로 사용하실 수 있습니다.")
    }
}

@Composable
private fun VolumeAdjustmentStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxWidth().align(Alignment.TopCenter),
            painter = painterResource(R.drawable.img_volume_step2),
            contentScale = ContentScale.FillWidth,
            contentDescription = "무선 마이크 수신기 볼륨 조절"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "무선 마이크 수신기에서 음량을 추가로 조절하실 수 있습니다.")
    }
}

@Composable
private fun VolumeAdjustmentStep3() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_volume_step1),
            contentScale = ContentScale.Crop,
            contentDescription = "믹서 볼륨 조절"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.3f, 0.45f).rotate(-90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.3f, 0.45f, xOffset = 48.dp, yOffset = 48.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "개별 마이크 음량")

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.7f, 0.35f, xOffset = (-48).dp, yOffset = (-48).dp).rotate(90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.7f, 0.35f, xOffset = (-180).dp, yOffset = (-105).dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "전체 음량")

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.85f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "믹서의 음량을 조절해 주세요.\n단, 믹서의 음량을 너무 키우면 하울링이 자주 발생하게 되니 유의해 주세요.")
    }
}

@Composable
fun VolumeAdjustment(onClose: () -> Unit) {
    var page by remember { mutableStateOf(0) }

    Column(modifier = Modifier.fillMaxSize()) {
        Box(modifier = Modifier.weight(1.0f)) {
            when (page) {
                0 -> VolumeAdjustmentStep1()
                1 -> VolumeAdjustmentStep2()
                2 -> VolumeAdjustmentStep3()
            }
        }
        Column {
            Row(modifier = Modifier.fillMaxWidth().height(64.dp)) {
                TextButton(
                    modifier = Modifier.weight(1.0f).fillMaxHeight(),
                    enabled = page > 0,
                    onClick = { if (page > 0) page -= 1 },
                    colors = ButtonDefaults.textButtonColors(contentColor = Color.Black)
                ) {
                    Text(
                        modifier = Modifier.padding(start = 16.dp),
                        style = MaterialTheme.typography.headlineSmall,
                        text = "이전"
                    )
                }
                VerticalDivider()
                TextButton(
                    modifier = Modifier.weight(1.0f).fillMaxHeight(),
                    onClick = {
                        if (page < MAX_PAGE_INDEX) {
                            page += 1
                        } else {
                            onClose()
                        }
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = Color.Black)
                ) {
                    Text(
                        modifier = Modifier.padding(start = 16.dp),
                        style = MaterialTheme.typography.headlineSmall,
                        text = if (page == MAX_PAGE_INDEX) "확인" else "다음"
                    )
                }
            }
        }
    }
}
