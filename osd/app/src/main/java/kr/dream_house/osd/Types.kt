package kr.dream_house.osd

import kotlinx.serialization.Serializable

@Serializable
enum class AlertSeverity {
    INTRUSIVE,
    URGENT,
    NORMAL,
}

typealias BookingId = String
typealias UnitId = String
