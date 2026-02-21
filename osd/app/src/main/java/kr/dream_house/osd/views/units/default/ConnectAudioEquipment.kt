package kr.dream_house.osd.views.units.default

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
private fun ConnectAudioEquipmentStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_connect_audio_equipment),
            contentScale = ContentScale.Crop,
            contentDescription = "케이블"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.1f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "외부 음향 장비를 연결하려면 이 케이블을 사용해 주세요.\n케이블은 입구 쪽에 걸려 있습니다.")
    }
}

@Composable
private fun ConnectAudioEquipmentStep2() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_connect_mixer),
            contentScale = ContentScale.Crop,
            contentDescription = "믹서 연결"
        )

        IndicatorArrow(modifier = Modifier.fractionalOffset(0.45f, 0.57f).rotate(-90.0f))
        Text(
            modifier = Modifier.fractionalOffset(0.45f, 0.57f, xOffset = 48.dp, yOffset = 48.dp)
                .background(LabelBackground).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.headlineSmall,
            text = "9/10 채널에 연결해주세요")
    }
}

@Composable
fun ConnectAudioEquipment(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { ConnectAudioEquipmentStep1() },
            { ConnectAudioEquipmentStep2() },
        ),
        onClose = onClose,
    )
}
