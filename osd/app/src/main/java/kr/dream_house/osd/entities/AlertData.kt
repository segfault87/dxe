package kr.dream_house.osd.entities

import kotlinx.serialization.Serializable
import kr.dream_house.osd.AlertSeverity

@Serializable
data class AlertData(
    val severity: AlertSeverity,
    val title: String,
    val contents: String,
    val closeable: Boolean,
)