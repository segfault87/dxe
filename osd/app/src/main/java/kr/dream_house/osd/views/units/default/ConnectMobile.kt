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
private fun ConnectMobileStep1() {
    Box(modifier = Modifier.fillMaxSize()) {
        Image(
            modifier = Modifier.fillMaxSize(),
            painter = painterResource(R.drawable.img_connect_mobile),
            contentScale = ContentScale.Crop,
            contentDescription = "케이블"
        )

        Text(
            modifier = Modifier.fractionalOffset(0.05f, 0.80f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "휴대폰을 믹서에 연결하려면 이 케이블을 사용해 주세요.\nUSB-C만 지원하며, 라이트닝 단자는 지원하지 않습니다.\n케이블은 입구 쪽에 걸려 있습니다.")
    }
}

@Composable
private fun ConnectMobileStep2() {
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
fun ConnectMobile(onClose: () -> Unit) {
    InstructionPage(
        pages = listOf(
            { ConnectMobileStep1() },
            { ConnectMobileStep2() },
        ),
        onClose = onClose,
    )
}
