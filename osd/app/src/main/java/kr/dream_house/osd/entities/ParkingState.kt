package kr.dream_house.osd.entities

import kotlinx.serialization.Serializable
import kotlin.time.ExperimentalTime
import kotlin.time.Instant

@OptIn(ExperimentalTime::class)
@Serializable
data class ParkingState(
    val licensePlateNumber: String,
    val userName: String,
    val entryDate: Instant,
    val exempted: Boolean,
    val fuzzy: String?,
)