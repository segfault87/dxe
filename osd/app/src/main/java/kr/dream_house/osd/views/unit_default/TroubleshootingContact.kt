package kr.dream_house.osd.views.unit_default

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
import kr.dream_house.osd.R

@Composable
fun TroubleshootingContact(modifier: Modifier = Modifier, message: String) {
    Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Image(
                modifier = Modifier.size(320.dp),
                painter = painterResource(R.drawable.img_qr_telephone),
                contentDescription = "Mobile number (+82-502-1944-5052)"
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