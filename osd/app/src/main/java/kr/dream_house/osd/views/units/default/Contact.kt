package kr.dream_house.osd.views.units.default

import androidx.compose.runtime.Composable
import kr.dream_house.osd.views.TroubleshootingContact

@Composable
fun Contact(onClose: () -> Unit) {
    TroubleshootingContact(message = "시설 이용 중 문제가 있거나 궁금하신 점이 있다면\n위 연락처로 문의주시기 바랍니다.")
}
