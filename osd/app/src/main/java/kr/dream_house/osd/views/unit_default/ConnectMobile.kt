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
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.ui.theme.LabelBackground
import kr.dream_house.osd.utils.fractionalOffset
import kr.dream_house.osd.views.IndicatorArrow

private const val MAX_PAGE_INDEX = 1

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
            modifier = Modifier.fractionalOffset(0.05f, 0.82f)
                .background(Color.White).padding(horizontal = 16.dp, vertical = 8.dp),
            style = MaterialTheme.typography.bodyLarge,
            text = "휴대폰을 믹서에 연결하려면 이 케이블을 사용해 주세요.\nUSB-C만 지원하며, 라이트닝 단자는 지원하지 않습니다.")
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
    var page by remember { mutableStateOf(0) }

    Column(modifier = Modifier.fillMaxSize()) {
        Box(modifier = Modifier.weight(1.0f)) {
            when (page) {
                0 -> ConnectMobileStep1()
                1 -> ConnectMobileStep2()
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
