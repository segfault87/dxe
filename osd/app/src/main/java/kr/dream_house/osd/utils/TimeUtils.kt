package kr.dream_house.osd.utils

import kotlin.time.Duration

fun Duration.format(): String {
    val hours = inWholeHours
    val minutes = inWholeMinutes - hours * 60

    var result = ""
    if (hours > 0) {
        result += "${hours}시간"
    }
    if (minutes >= 0) {
        if (result.isNotEmpty()) {
            result += " "
        }
        result += "${minutes}분"
    }

    return result
}