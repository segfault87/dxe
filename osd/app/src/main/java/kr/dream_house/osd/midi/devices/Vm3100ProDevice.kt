package kr.dream_house.osd.midi.devices

import kr.dream_house.osd.midi.ChannelControlParameter
import kr.dream_house.osd.midi.ControlValue
import kr.dream_house.osd.midi.GlobalControlParameter
import kr.dream_house.osd.midi.MixerDevice
import kotlin.experimental.and
import kotlin.experimental.or
import kotlin.math.ceil

class Vm3100ProMixerDevice : MixerDevice {
    companion object {
        private val CHANNEL_MAPPINGS = arrayOf(
            arrayOf(2),
            arrayOf(3),
            arrayOf(10, 11),
            arrayOf(8, 9),
            arrayOf(0),
            arrayOf(1),
            arrayOf(4),
            arrayOf(5),
            arrayOf(6),
            arrayOf(7)
        )

        private val CHANNEL_REVERSE_MAPPINGS = mapOf(
            2 to 0,
            3 to 1,
            10 to 2,
            8 to 3,
            0 to 4,
            1 to 5,
            4 to 6,
            5 to 7,
            6 to 8,
            7 to 9,
        )

        private val EQ_GAIN_TABLE = mapOf<Int, Byte>(
            -12 to 0,
            -11 to 5,
            -10 to 10,
            -9 to 15,
            -8 to 20,
            -7 to 25,
            -6 to 30,
            -5 to 35,
            -4 to 40,
            -3 to 45,
            -2 to 50,
            -1 to 55,
            0 to 64,
            1 to 72,
            2 to 77,
            3 to 82,
            4 to 87,
            5 to 92,
            6 to 97,
            7 to 102,
            8 to 107,
            9 to 112,
            10 to 117,
            11 to 122,
            12 to 127,
        )
        private val EQ_GAIN_REVERSE_TABLE =
            EQ_GAIN_TABLE.entries.associate { (key, value) -> value to key }

        private val EQ_HIGH_MAPPINGS = listOf<Pair<Float, Byte>>(
            400.0f to 0,
            425.0f to 2,
            452.0f to 4,
            481.0f to 6,
            512.0f to 8,
            545.0f to 10,
            580.0f to 12,
            617.0f to 14,
            657.0f to 16,
            699.0f to 18,
            744.0f to 20,
            791.0f to 22,
            842.0f to 24,
            896.0f to 26,
            954.0f to 28,
            1020.0f to 30,
            1080.0f to 32,
            1150.0f to 34,
            1220.0f to 36,
            1300.0f to 38,
            1380.0f to 40,
            1470.0f to 42,
            1570.0f to 44,
            1670.0f to 46,
            1780.0f to 48,
            1890.0f to 50,
            2010.0f to 52,
            2140.0f to 54,
            2280.0f to 56,
            2420.0f to 58,
            2580.0f to 60,
            2740.0f to 62,
            2920.0f to 64,
            3100.0f to 66,
            3300.0f to 68,
            3520.0f to 70,
            3740.0f to 72,
            3980.0f to 74,
            4230.0f to 76,
            4510.0f to 78,
            4790.0f to 80,
            5100.0f to 82,
            5430.0f to 84,
            5780.0f to 86,
            6150.0f to 88,
            6540.0f to 90,
            6960.0f to 92,
            7410.0f to 94,
            7880.0f to 96,
            8380.0f to 98,
            8920.0f to 100,
            9490.0f to 102,
            10100.0f to 104,
            10750.0f to 106,
            11440.0f to 108,
            12170.0f to 110,
            12950.0f to 112,
            13780.0f to 114,
            14660.0f to 116,
            15600.0f to 118,
            16600.0f to 120,
            17660.0f to 122,
            18800.0f to 124,
            20000.0f to 126,
        )
        private val EQ_HIGH_REVERSE_MAPPINGS =
            EQ_HIGH_MAPPINGS.associate { (key, value) -> value to key }

        private val EQ_MID_MAPPINGS = listOf<Pair<Float, Byte>>(
            200.0f to 0,
            212.0f to 2,
            224.0f to 4,
            238.0f to 6,
            252.0f to 8,
            268.0f to 10,
            284.0f to 12,
            301.0f to 14,
            319.0f to 16,
            338.0f to 18,
            359.0f to 20,
            380.0f to 22,
            403.0f to 24,
            428.0f to 26,
            453.0f to 28,
            481.0f to 30,
            510.0f to 32,
            541.0f to 34,
            573.0f to 36,
            608.0f to 38,
            645.0f to 40,
            683.0f to 42,
            725.0f to 44,
            768.0f to 46,
            815.0f to 48,
            864.0f to 50,
            916.0f to 52,
            971.0f to 54,
            1030.0f to 56,
            1090.0f to 58,
            1160.0f to 60,
            1230.0f to 62,
            1300.0f to 64,
            1380.0f to 66,
            1460.0f to 68,
            1550.0f to 70,
            1650.0f to 72,
            1750.0f to 74,
            1850.0f to 76,
            1960.0f to 78,
            2080.0f to 80,
            2210.0f to 82,
            2340.0f to 84,
            2480.0f to 86,
            2630.0f to 88,
            2790.0f to 90,
            2960.0f to 92,
            3130.0f to 94,
            3320.0f to 96,
            3520.0f to 98,
            3740.0f to 100,
            3960.0f to 102,
            4200.0f to 104,
            4450.0f to 106,
            4720.0f to 108,
            5010.0f to 110,
            5310.0f to 112,
            5630.0f to 114,
            5970.0f to 116,
            6330.0f to 118,
            6710.0f to 120,
            7120.0f to 122,
            7550.0f to 124,
            8000.0f to 126,
        )
        private val EQ_MID_REVERSE_MAPPINGS =
            EQ_MID_MAPPINGS.associate { (key, value) -> value to key }

        private val EQ_LOW_MAPPINGS = listOf<Pair<Float, Byte>>(
            20.0f to 0,
            21.0f to 2,
            23.0f to 4,
            24.0f to 6,
            26.0f to 8,
            28.0f to 10,
            31.0f to 12,
            33.0f to 14,
            35.0f to 16,
            38.0f to 18,
            41.0f to 20,
            44.0f to 22,
            48.0f to 24,
            51.0f to 26,
            55.0f to 28,
            59.0f to 30,
            64.0f to 32,
            69.0f to 34,
            74.0f to 36,
            80.0f to 38,
            86.0f to 40,
            92.0f to 42,
            99.0f to 44,
            107.0f to 46,
            115.0f to 48,
            124.0f to 50,
            133.0f to 52,
            143.0f to 54,
            154.0f to 56,
            166.0f to 58,
            179.0f to 60,
            192.0f to 62,
            207.0f to 64,
            223.0f to 66,
            240.0f to 68,
            258.0f to 70,
            277.0f to 72,
            298.0f to 74,
            321.0f to 76,
            346.0f to 78,
            372.0f to 80,
            400.0f to 82,
            430.0f to 84,
            463.0f to 86,
            498.0f to 88,
            536.0f to 90,
            577.0f to 92,
            621.0f to 94,
            668.0f to 96,
            718.0f to 98,
            773.0f to 100,
            831.0f to 102,
            895.0f to 104,
            962.0f to 106,
            1040.0f to 108,
            1110.0f to 110,
            1200.0f to 112,
            1290.0f to 114,
            1390.0f to 116,
            1490.0f to 118,
            1610.0f to 120,
            1730.0f to 122,
            1860.0f to 124,
            2000.0f to 126,
        )
        private val EQ_LOW_REVERSE_MAPPINGS =
            EQ_LOW_MAPPINGS.associate { (key, value) -> value to key }

        private val EQ_MID_Q_MAPPINGS = listOf<Pair<Float, Byte>>(
            0.5f to 0,
            1.0f to 21,
            2.0f to 42,
            4.0f to 63,
            8.0f to 84,
        )
        private val EQ_MID_Q_REVERSE_MAPPINGS =
            EQ_MID_Q_MAPPINGS.associate { (key, value) -> value to key }

        private val FIXED_DEFAULT_PAYLOADS = listOf(
            // Master balance (center)
            byteArrayOf(0xbf.toByte(), 70, 64),
            // Monitor balance (center)
            byteArrayOf(0xbf.toByte(), 72, 64),
            // FX1 (reverb) level
            byteArrayOf(0xbf.toByte(), 73, 100),
            // AUX level
            byteArrayOf(0xbf.toByte(), 75, 127),
            // AUX balance
            byteArrayOf(0xbf.toByte(), 76, 64),
            // BUS level
            byteArrayOf(0xbf.toByte(), 77, 127),
            // BUS balance
            byteArrayOf(0xbf.toByte(), 78, 64),
            // D-out (A) level
            byteArrayOf(0xbf.toByte(), 79, 127),
            // D-out (A) balance
            byteArrayOf(0xbf.toByte(), 80, 64),
            // D-out (B) level
            byteArrayOf(0xbf.toByte(), 81, 127),
            // D-out (B) balance
            byteArrayOf(0xbf.toByte(), 82, 64),
        )
    }

    override val channelNames: Array<String>
        get() = arrayOf("무선마이크 1", "무선마이크 2", "건반 (채널 11/12)", "채널 9/10", "채널 1", "채널 2", "채널 5", "채널 6", "채널 7", "채널 8")

    private fun tableSearch(value: Float, table: List<Pair<Float, Byte>>): Byte {
        val index = table.binarySearchBy(value) { it.first }

        return when {
            index >= 0 -> table[index].second
            index == -1 -> table.first().second
            else -> {
                val insertionPoint = -(index + 1)
                if (insertionPoint >= table.size) {
                    table.last().second
                } else {
                    table[insertionPoint - 1].second
                }
            }
        }
    }

    override fun translateChannelLevelValue(level: Float): Byte {
        return (level.coerceIn(0.0f, 1.0f) * 127.0f).toInt().toByte()
    }

    override fun translateRemoteChannelLevelValue(value: Byte): Float {
        return value.toFloat() / 127.0f
    }

    override fun translateChannelPanValue(pan: Float): Byte {
        return ceil((pan.coerceIn(-1.0f, 1.0f) + 1.0f) * 0.5f * 127.0f).toInt().toByte()
    }

    override fun translateRemoteChannelPanValue(value: Byte): Float {
        return value.toFloat() / 127.0f * 2.0f - 1.0f;
    }

    override fun translateChannelReverbValue(reverb: Float): Byte {
        return (reverb.coerceIn(0.0f, 1.0f) * 127.0f).toInt().toByte()
    }

    override fun translateRemoteChannelReverbValue(value: Byte): Float {
        return value.toFloat() / 127.0f
    }

    override fun translateChannelMuteValue(mute: Boolean): Byte {
        return if (mute) {
            1
        } else {
            0
        }
    }

    override fun translateRemoteChannelMuteValue(value: Byte): Boolean? {
        return when (value) {
            0.toByte() -> false
            1.toByte() -> true
            else -> null
        }
    }

    override fun translateChannelEqLevelValue(level: Float): Byte? {
        return EQ_GAIN_TABLE[level.coerceIn(-12.0f, 12.0f).toInt()]
    }

    override fun translateRemoteChannelEqLevelValue(value: Byte): Float? {
        return EQ_GAIN_REVERSE_TABLE[value]?.toFloat()
    }

    override fun translateChannelThreeBandEqHighFreqValue(freq: Float): Byte {
        return tableSearch(freq, EQ_HIGH_MAPPINGS)
    }

    override fun translateRemoteChannelThreeBandEqHighFreqValue(value: Byte): Float? {
        return EQ_HIGH_REVERSE_MAPPINGS[value]
    }

    override fun translateChannelThreeBandEqMidFreqValue(freq: Float): Byte {
        return tableSearch(freq, EQ_MID_MAPPINGS)
    }

    override fun translateRemoteChannelThreeBandEqMidFreqValue(value: Byte): Float? {
        return EQ_MID_REVERSE_MAPPINGS[value]
    }

    override fun translateChannelThreeBandEqLowFreqValue(freq: Float): Byte {
        return tableSearch(freq, EQ_LOW_MAPPINGS)
    }

    override fun translateRemoteChannelThreeBandEqLowFreqValue(value: Byte): Float? {
        return EQ_LOW_REVERSE_MAPPINGS[value]
    }

    override fun translateChannelThreeBandEqMidQValue(q: Float): Byte {
        return tableSearch(q, EQ_MID_Q_MAPPINGS)
    }

    override fun translateRemoteChannelThreeBandEqMidQValue(value: Byte): Float? {
        return EQ_MID_Q_REVERSE_MAPPINGS[value]
    }

    override fun translateGlobalMasterLevelValue(level: Float): Byte {
        return (level.coerceIn(0.0f, 1.0f) * 127.0f).toInt().toByte()
    }

    override fun translateRemoteGlobalMasterLevelValue(value: Byte): Float {
        return value.toFloat() / 127.0f
    }

    override fun translateGlobalMonitorLevelValue(level: Float): Byte {
        return (level.coerceIn(0.0f, 1.0f) * 127.0f).toInt().toByte()
    }

    override fun translateRemoteGlobalMonitorLevelValue(value: Byte): Float {
        return value.toFloat() / 127.0f
    }

    private fun getChannelParameterNumber(parameter: ChannelControlParameter, channel: Byte): Byte? {
        return if ((0..15).contains(channel)) {
            when (parameter) {
                ChannelControlParameter.MUTE -> 25
                ChannelControlParameter.LEVEL -> 7
                ChannelControlParameter.REVERB -> 19
                ChannelControlParameter.PAN -> 10
                ChannelControlParameter.EQ_LOW_FREQ -> 12
                ChannelControlParameter.EQ_LOW_LEVEL -> 13
                ChannelControlParameter.EQ_MID_FREQ -> 14
                ChannelControlParameter.EQ_MID_LEVEL -> 15
                ChannelControlParameter.EQ_MID_Q -> 16
                ChannelControlParameter.EQ_HIGH_FREQ -> 17
                ChannelControlParameter.EQ_HIGH_LEVEL -> 18
            }
        } else if ((16..19).contains(channel)) {
            when (parameter) {
                ChannelControlParameter.MUTE -> 84
                ChannelControlParameter.LEVEL -> 68
                ChannelControlParameter.REVERB -> 78
                ChannelControlParameter.PAN -> 70
                ChannelControlParameter.EQ_LOW_FREQ -> 71
                ChannelControlParameter.EQ_LOW_LEVEL -> 72
                ChannelControlParameter.EQ_MID_FREQ -> 73
                ChannelControlParameter.EQ_MID_LEVEL -> 74
                ChannelControlParameter.EQ_MID_Q -> 75
                ChannelControlParameter.EQ_HIGH_FREQ -> 76
                ChannelControlParameter.EQ_HIGH_LEVEL -> 77
            }
        } else {
            null
        }
    }

    private fun getGlobalParameterNumber(parameter: GlobalControlParameter): Byte {
        return when (parameter) {
            GlobalControlParameter.MASTER_LEVEL -> 68
            GlobalControlParameter.MONITOR_LEVEL -> 71
        }
    }

    private fun parseCCValue(parameter: Byte, channel: Byte, value: Byte): ControlValue? {
        if (channel == 0x0f.toByte()) {
            when (parameter.toInt()) {
                68 -> return ControlValue.GlobalValue(
                    control = GlobalControlParameter.MASTER_LEVEL,
                    value = value,
                )
                71 -> return ControlValue.GlobalValue(
                    control = GlobalControlParameter.MONITOR_LEVEL,
                    value = value,
                )
            }
        }

        val (control, base) = when (parameter.toInt()) {
            25 -> ChannelControlParameter.MUTE to 0
            7 -> ChannelControlParameter.LEVEL to 0
            19 -> ChannelControlParameter.REVERB to 0
            10 -> ChannelControlParameter.PAN to 0
            12 -> ChannelControlParameter.EQ_LOW_FREQ to 0
            13 -> ChannelControlParameter.EQ_LOW_LEVEL to 0
            14 -> ChannelControlParameter.EQ_MID_FREQ to 0
            15 -> ChannelControlParameter.EQ_MID_LEVEL to 0
            16 -> ChannelControlParameter.EQ_MID_Q to 0
            17 -> ChannelControlParameter.EQ_HIGH_FREQ to 0
            18 -> ChannelControlParameter.EQ_HIGH_LEVEL to 0
            84 -> ChannelControlParameter.MUTE to 16
            68 -> ChannelControlParameter.LEVEL to 16
            78 -> ChannelControlParameter.REVERB to 16
            70 -> ChannelControlParameter.PAN to 16
            71 -> ChannelControlParameter.EQ_LOW_FREQ to 16
            72 -> ChannelControlParameter.EQ_LOW_LEVEL to 16
            73 -> ChannelControlParameter.EQ_MID_FREQ to 16
            74 -> ChannelControlParameter.EQ_MID_LEVEL to 16
            75 -> ChannelControlParameter.EQ_MID_Q to 16
            76 -> ChannelControlParameter.EQ_HIGH_FREQ to 16
            77 -> ChannelControlParameter.EQ_HIGH_LEVEL to 16
            else -> return null
        }
        val channelSeq = CHANNEL_REVERSE_MAPPINGS[channel.toInt() + base] ?: return null

        return ControlValue.ChannelValue(
            control = control,
            channel = channelSeq,
            value = value,
        )
    }

    override fun initializeState(initialStates: List<ControlValue>): List<ByteArray> {
        val buffers = mutableListOf<ByteArray>()

        for (value in initialStates) {
            val buffer = ByteArray(getCCPayloadSizeHint(value))
            buildCCPayload(value, buffer, 0)
            buffers.add(buffer)
        }

        for (payload in FIXED_DEFAULT_PAYLOADS) {
            buffers.add(payload)
        }

        return buffers
    }

    override fun parseCCPayload(packet: ByteArray, offset: Int, size: Int): ControlValue? {
        if (size != 3) {
            return null
        }

        val status = packet[offset]
        if (status and 0xB0.toByte() != 0xB0.toByte()) {
            return null
        }

        val channel = status and 0x0F.toByte()

        return parseCCValue(packet[offset + 1], channel, packet[offset + 2])
    }

    override fun buildCCPayload(value: ControlValue, output: ByteArray, offset: Int): Int {
        return when (value) {
            is ControlValue.ChannelValue -> {
                val channels = CHANNEL_MAPPINGS[value.channel]

                var count = 0

                channels.forEachIndexed { index, channel ->
                    val channel = channel.toByte()
                    val parameterNumber = getChannelParameterNumber(value.control, channel) ?: return@forEachIndexed

                    output[offset + index * 3] = 0xb0.toByte() or (channel and 0xf)
                    output[offset + index * 3 + 1] = parameterNumber
                    output[offset + index * 3 + 2] = value.value

                    count += 3
                }

                count
            }
            is ControlValue.GlobalValue -> {
                val parameterNumber = getGlobalParameterNumber(value.control)

                output[offset] = 0xbf.toByte()
                output[offset + 1] = parameterNumber
                output[offset + 2] = value.value

                3
            }
        }
    }

    override fun getCCPayloadSizeHint(value: ControlValue): Int {
        return when (value) {
            is ControlValue.ChannelValue -> {
                val channels = CHANNEL_MAPPINGS[value.channel]
                channels.size * 3
            }
            is ControlValue.GlobalValue -> {
                3
            }
        }
    }

    override fun flowControlMilliseconds(): Long = 10

    override fun maxPayloadInBatch(): Int = 5
}