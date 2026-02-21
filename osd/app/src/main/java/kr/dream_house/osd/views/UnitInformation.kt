package kr.dream_house.osd.views

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.R
import kr.dream_house.osd.views.units.PAGES_PER_UNIT

data class Subpage(
    val title: String,
    val contents: @Composable (onClose: () -> Unit) -> Unit,
)

@Composable
private fun Subpage.MenuItemButton(onClick: () -> Unit) {
    Button(modifier = Modifier.width(520.dp), onClick = onClick) {
        Text(modifier = Modifier.padding(16.dp), text = title, style = MaterialTheme.typography.bodyLarge)
    }
}

@Composable
fun UnitInformation() {
    val subpages = PAGES_PER_UNIT[BuildConfig.UNIT_ID] ?: return

    var currentPage by remember { mutableStateOf<Subpage?>(null) }

    if (currentPage != null) {
        Box(modifier = Modifier.fillMaxSize()) {
            currentPage?.contents({ currentPage = null })
            IconButton(modifier = Modifier.padding(16.dp).align(Alignment.TopEnd), onClick = { currentPage = null }) {
                Image(painter = painterResource(R.drawable.ic_close), contentDescription = "Back")
            }
        }
    } else {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                for (subpage in subpages) {
                    subpage.MenuItemButton {
                        currentPage = subpage
                    }
                }
            }
        }
    }
}
