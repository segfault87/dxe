package kr.dream_house.osd.views

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.views.units.CONTACT_PER_UNIT

@Composable
fun TroubleshootingContact(modifier: Modifier = Modifier, message: String) {
    val contactResource = CONTACT_PER_UNIT[BuildConfig.UNIT_ID] ?: return

    Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Image(
                modifier = Modifier.size(320.dp),
                painter = painterResource(contactResource),
                contentDescription = "Mobile number"
            )
            Text(
                modifier = Modifier.padding(top = 32.dp),
                textAlign = TextAlign.Center,
                text = message,
                style = MaterialTheme.typography.bodyLarge,
            )
        }
    }
}
