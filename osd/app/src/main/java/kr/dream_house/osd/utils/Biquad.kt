package kr.dream_house.osd.utils

import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.log10
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt

private const val SAMPLING_RATE: Double = 48000.0

private data class Biquad(
    val b0: Double,
    val b1: Double,
    val b2: Double,
    val a1: Double,
    val a2: Double,
) {
    companion object {
        fun normalize(b0: Double, b1: Double, b2: Double, a0: Double, a1: Double, a2: Double): Biquad {
            return Biquad(
                b0 = b0 / a0, b1 = b1 / a0, b2 = b2 / a0, a1 = a1 / a0, a2 = a2 / a0
            )
        }
    }
}

private fun Biquad.tx(z: Float): Float {
    val phi = (sin(2.0f * PI * z / (2.0f * SAMPLING_RATE))).pow(2.0)
    val r = (
        (b0 + b1 + b2).pow(2.0)
        - 4.0 * (b0 * b1 + 4.0 * b0 * b2 + b1 * b2) * phi
        + 16.0 * b0 * b2 * phi * phi
    ) / (
        (1.0 + a1 + a2).pow(2.0)
        - 4.0 * (a1 + 4.0 * a2 + a1 * a2) * phi
        + 16.0 * a2 * phi * phi
    )

    return sqrt(if (r < 0.0) { 0.0 } else { r }).toFloat()
}

private fun Biquad.gain(z: Float): Float {
    return 20.0f * log10(abs(tx(z)))
}

private data class TxParameters(
    val a: Double,
    val omega: Double,
    val alpha: Double,
) {
    companion object {
        fun peak(f: Double, gain: Double, q: Double): TxParameters {
            val a = 10.0.pow(gain / 40.0)
            val omega = 2.0 * PI * (f / SAMPLING_RATE)
            val alpha = sin(omega) / (2.0 * q)
            return TxParameters(a, omega, alpha)
        }

        fun shelf(f: Double, gain: Double): TxParameters {
            val a = 10.0.pow(gain / 40.0)
            val omega = 2.0 * PI * (f / SAMPLING_RATE)
            val alpha = sin(omega) / 2.0 * sqrt(2.0)
            return TxParameters(a, omega, alpha)
        }
    }
}

private object BiquadFunctions {
    fun lowShelf(f: Double, gain: Double): Biquad {
        val p = TxParameters.shelf(f, gain)
        val b0 = p.a * ((p.a + 1.0) - (p.a - 1.0) * cos(p.omega) + 2.0 * sqrt(p.a) * p.alpha)
        val b1 = 2.0 * p.a * ((p.a - 1.0) - (p.a + 1.0) * cos(p.omega))
        val b2 = p.a * ((p.a + 1.0) - (p.a - 1.0) * cos(p.omega) - 2.0 * sqrt(p.a) * p.alpha)
        val a0 = (p.a + 1.0) + (p.a - 1.0) * cos(p.omega) + 2.0 * sqrt(p.a) * p.alpha
        val a1 = -2.0 * ((p.a - 1.0) + (p.a + 1.0) * cos(p.omega))
        val a2 = (p.a + 1.0) + (p.a - 1.0) * cos(p.omega) - 2.0 * sqrt(p.a) * p.alpha
        return Biquad.normalize(b0, b1, b2, a0, a1, a2)
    }

    fun midPeak(f: Double, gain: Double, q: Double): Biquad {
        val p = TxParameters.peak(f, gain, q)
        val b0 = 1.0 + (p.alpha * p.a)
        val b1 = -2.0 * cos(p.omega)
        val b2 = 1.0 - (p.alpha * p.a)
        val a0 = 1.0 + (p.alpha / p.a)
        val a1 = -2.0 * cos(p.omega)
        val a2 = 1.0 - (p.alpha / p.a)
        return Biquad.normalize(b0, b1, b2, a0, a1, a2)
    }

    fun highShelf(f: Double, gain: Double): Biquad {
        val p = TxParameters.shelf(f, gain)
        val b0 = p.a * ((p.a + 1.0) + (p.a - 1.0) * cos(p.omega) + 2.0 * sqrt(p.a) * p.alpha)
        val b1 = -2.0 * p.a * ((p.a - 1.0) + (p.a + 1.0) * cos(p.omega))
        val b2 = p.a * ((p.a + 1.0) + (p.a - 1.0) * cos(p.omega) - 2.0 * sqrt(p.a) * p.alpha)
        val a0 = (p.a + 1.0) - (p.a - 1.0) * cos(p.omega) + 2.0 * sqrt(p.a) * p.alpha
        val a1 = 2.0 * ((p.a - 1.0) - (p.a + 1.0) * cos(p.omega))
        val a2 = (p.a + 1.0) - (p.a - 1.0) * cos(p.omega) - 2.0 * sqrt(p.a) * p.alpha
        return Biquad.normalize(b0, b1, b2, a0, a1, a2)
    }
}

class ThreeBandEq(lowF: Float, lowGain: Float, midF: Float, midGain: Float, midQ: Float, highF: Float, highGain: Float) {
    private var low = BiquadFunctions.lowShelf(lowF.toDouble(), lowGain.toDouble())
    private var mid = BiquadFunctions.midPeak(midF.toDouble(), midGain.toDouble(), midQ.toDouble())
    private var high = BiquadFunctions.highShelf(highF.toDouble(), highGain.toDouble())

    fun updateLow(f: Float, gain: Float) {
        low = BiquadFunctions.lowShelf(f.toDouble(), gain.toDouble())
    }

    fun updateMid(f: Float, gain: Float, q: Float) {
        mid = BiquadFunctions.midPeak(f.toDouble(), gain.toDouble(), q.toDouble())
    }

    fun updateHigh(f: Float, gain: Float) {
        high = BiquadFunctions.highShelf(f.toDouble(), gain.toDouble())
    }

    fun calculateGain(z: Float): Float {
        return low.gain(z) + mid.gain(z) + high.gain(z)
    }
}