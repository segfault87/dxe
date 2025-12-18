package kr.dream_house.osd.views.unit_default

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
import kr.dream_house.osd.R

enum class Subpage(val contents: @Composable (onClose: () -> Unit) -> Unit) {
    MIXER_SETUP({ MixerSetup(it) }),
    VOLUME_ADJUSTMENT({ VolumeAdjustment(it) }),
    REVERB_SETTINGS({ ReverbSettings(it) }),
    CONNECT_MOBILE({ ConnectMobile(it) }),
    CONNECT_AUDIO_EQUIPMENT({ ConnectAudioEquipment(it) }),
    CONTACT({ Contact(it) });
}

@Composable
fun MenuItemButton(onClick: () -> Unit, text: String) {
    Button(modifier = Modifier.width(420.dp), onClick = onClick) {
        Text(modifier = Modifier.padding(16.dp), text = text, style = MaterialTheme.typography.bodyLarge)
    }
}

// 896x680

@Composable
fun UnitInformation() {
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
                MenuItemButton(
                    onClick = { currentPage = Subpage.MIXER_SETUP },
                    text = "마이크, 건반 소리가 안 나요"
                )
                MenuItemButton(
                    onClick = { currentPage = Subpage.VOLUME_ADJUSTMENT },
                    text = "마이크 소리가 잘 안 들려요"
                )
                MenuItemButton(
                    onClick = { currentPage = Subpage.REVERB_SETTINGS },
                    text = "마이크에 리버브를 걸고 싶어요"
                )
                MenuItemButton(
                    onClick = { currentPage = Subpage.CONNECT_MOBILE },
                    text = "휴대폰에 있는 음악을 재생하고 싶어요"
                )
                MenuItemButton(
                    onClick = { currentPage = Subpage.CONNECT_AUDIO_EQUIPMENT },
                    text = "건반을 추가로 연결하고 싶어요"
                )
                MenuItemButton(onClick = { currentPage = Subpage.CONTACT }, text = "그 외 문의사항이 있어요")
            }
        }
    }
}