package kr.dream_house.osd.views.units.default

import kr.dream_house.osd.R
import kr.dream_house.osd.views.Subpage

val UNIT_DEFAULT_PAGES = listOf(
    Subpage(title = "모니터링이 잘 안 돼요 / 마이크 소리가 너무 작아요", contents = { VolumeAdjustment(it) }),
    Subpage(title = "개인 모니터를 연결하고 싶어요.", contents = { PersonalMonitor(onClose = it) }),
    Subpage(title = "유선 마이크를 연결하고 싶어요.", contents = { WiredMic(onClose = it) }),
    Subpage(title = "휴대폰에 있는 음악을 재생하고 싶어요", contents = { ConnectMobile(it) }),
    Subpage(title = "건반을 추가로 연결하고 싶어요", contents = { ConnectAudioEquipment(it) }),
    Subpage(title = "그 외 문의사항이 있어요", contents = { Contact(it) }),
)

val CONTACT_INFORMATION_DEFAULT = R.drawable.img_qr_telephone_default
