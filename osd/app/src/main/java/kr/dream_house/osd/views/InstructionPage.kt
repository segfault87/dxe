package kr.dream_house.osd.views

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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

@Composable
fun InstructionPage(
    pages: List<@Composable () -> Unit>,
    onClose: () -> Unit,
) {
    var page by remember { mutableStateOf(0) }

    Column(modifier = Modifier.fillMaxSize()) {
        Box(modifier = Modifier.weight(1.0f)) {
            pages[page]()
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
                        if (page < pages.size -1) {
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
                        text = if (page == pages.size - 1) "확인" else "다음"
                    )
                }
            }
        }
    }
}