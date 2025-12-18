package kr.dream_house.osd.views

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.R
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.ui.theme.Pink40
import kr.dream_house.osd.utils.format
import kr.dream_house.osd.utils.timeRemainingFlow
import kotlin.time.Duration

@Composable
fun TopBar(
    modifier: Modifier = Modifier,
    activeBooking: Booking?,
) {
    Row(
        modifier = modifier.fillMaxWidth().height(120.dp).padding(horizontal = 32.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Image(
            modifier = Modifier.fillMaxHeight(),
            painter = painterResource(R.drawable.dxelogo),
            contentDescription = "Dream House Rehearsal Studio"
        )
        Spacer(Modifier.weight(1f))
        if (activeBooking != null) {
            val annotatedString = buildAnnotatedString {
                withStyle(style = SpanStyle(color = Pink40)) {
                    append("고객명 : ")
                }
                withStyle(style = SpanStyle(fontWeight = FontWeight.Bold)) {
                    append(activeBooking.customerName)
                }
            }
            Text(modifier = Modifier.padding(horizontal = 16.dp), text = annotatedString, style = MaterialTheme.typography.titleLarge)

            val remainingTime by activeBooking.timeRemainingFlow().collectAsState(Duration.ZERO)

            if (remainingTime.isPositive()) {
                val annotatedString = buildAnnotatedString {
                    withStyle(style = SpanStyle(color = Pink40)) {
                        append("잔여 시간 : ")
                    }
                    withStyle(style = SpanStyle(fontWeight = FontWeight.Bold)) {
                        append(remainingTime.format())
                    }
                }
                Text(modifier = Modifier.padding(horizontal = 16.dp), text = annotatedString, style = MaterialTheme.typography.titleLarge)
            }
        }
    }

}