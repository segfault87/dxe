package kr.dream_house.osd.midi

import kotlin.experimental.and
import kotlin.experimental.or

data class MidiNrpnParameter(
    val channel: Byte,
    val msb: Byte,
    val lsb: Byte,
    val coarse: Byte,
    val fine: Byte?,
) {
    companion object {
        val LENGTH_COARSE = 9
        val LENGTH_FINE = 12

        fun parse(buffer: ByteArray, offset: Int, size: Int): MidiNrpnParameter? {
            return if (size > 8) {
                if ((buffer[offset] and 0xb0.toByte() == 0xb0.toByte()) && buffer[offset + 1] == 0x63.toByte() && buffer[offset + 4] == 0x62.toByte()) {
                    val channel = buffer[offset] and 0x0f
                    val msb = buffer[offset + 2]
                    val lsb = buffer[offset + 5]

                    val coarse = buffer[offset + 8]
                    val fine: Byte?
                    if (buffer[offset + 7] == 0x06.toByte()) {
                        if (buffer[offset + 10] == 0x26.toByte()) {
                            fine = buffer[offset + 11]
                        } else {
                            throw IllegalArgumentException("Invalid fine CC value ${buffer[offset + 11]}")
                        }
                    } else if (buffer[offset + 7] == 0x60.toByte()) {
                        fine = null
                    } else {
                        throw IllegalArgumentException("Invalid coarse CC value ${buffer[offset + 7]}")
                    }

                    MidiNrpnParameter(
                        channel = channel,
                        msb = msb,
                        lsb = lsb,
                        coarse = coarse,
                        fine = fine,
                    )
                } else {
                    null
                }
            } else {
                null
            }
        }
    }

    fun buildInto(out: ByteArray, offset: Int): Int {
        val channel = 0xb0.toByte() or channel
        out[offset + 0] = channel
        out[offset + 1] = 0x63
        out[offset + 2] = msb
        out[offset + 3] = channel
        out[offset + 4] = 0x62
        out[offset + 5] = lsb
        out[offset + 6] = channel
        out[offset + 8] = coarse
        return if (fine == null) {
            out[offset + 7] = 0x60
            LENGTH_COARSE
        } else {
            out[offset + 7] = 0x06
            out[offset + 9] = channel
            out[offset + 10] = 0x26
            out[offset + 11] = fine
            LENGTH_FINE
        }
    }
}

data class MidiCCParameter(
    val channel: Byte,
    val parameter: Byte,
    val value: Byte,
) {
    companion object {
        const val LENGTH = 3

        fun parse(buffer: ByteArray, offset: Int): MidiCCParameter? {
            if (buffer.size - offset < 3) {
                return null
            }

            return MidiCCParameter(
                channel = buffer[offset] and 0x0f.toByte(),
                parameter = buffer[offset + 1],
                value = buffer[offset + 2],
            )
        }
    }

    fun buildInto(out: ByteArray, offset: Int) {
        out[offset] = 0xb0.toByte() or (channel and 0x0f)
        out[offset + 1] = parameter
        out[offset + 2] = value
    }
}
