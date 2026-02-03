package kr.dream_house.osd.entities

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BookingId
import kr.dream_house.osd.IdentityId
import kotlin.time.ExperimentalTime
import kotlin.time.Instant

@OptIn(ExperimentalTime::class)
@Serializable
data class Booking(
    val bookingId: BookingId,
    val customerId: IdentityId,
    val customerName: String,
    val timeFrom: Instant,
    val timeTo: Instant,
)