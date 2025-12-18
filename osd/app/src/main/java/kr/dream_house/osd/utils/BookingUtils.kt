package kr.dream_house.osd.utils

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.ParkingState
import kotlin.time.Clock
import kotlin.time.Duration
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
fun Booking.timeRemainingFlow(): Flow<Duration> = flow {
    while (true) {
        val now = Clock.System.now()
        emit(timeTo - now)
        delay(1000)
    }
}

@OptIn(ExperimentalTime::class)
fun ParkingState.elapsedTimeFlow(): Flow<Duration> = flow {
    while (true) {
        val now = Clock.System.now()
        emit(now - entryDate)
        delay(10000)
    }
}