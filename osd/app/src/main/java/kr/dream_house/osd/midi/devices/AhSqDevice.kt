package kr.dream_house.osd.midi.devices

import android.os.Bundle
import kr.dream_house.osd.midi.ChannelControlParameter
import kr.dream_house.osd.midi.ControlValue
import kr.dream_house.osd.midi.GlobalControlParameter
import kr.dream_house.osd.midi.MidiMixerSpec
import kr.dream_house.osd.midi.MidiNrpnParameter
import kr.dream_house.osd.midi.MixerConfigurations
import kr.dream_house.osd.midi.MixerDevice

// Interface for Allen & Heath SQ Mixers

class AhSqDevice : MixerDevice {
    override val spec = AhSqDevice

    companion object : MidiMixerSpec {
        override val identifier: String = "ah_sq"

        override fun probe(properties: Bundle): MixerDevice? {
            return if (properties.getString("manufacturer") == "Allen&Heath Ltd" && properties.getString("product") == "SQ") {
                AhSqDevice()
            } else {
                null
            }
        }

        private fun quantizeGainLevel(gain: Float): ByteArray {
            val gain = gain.coerceIn(-128.0f, 10.0f)

            val quantized = ((gain + 117.8f) / 128.0f * 16255.0f).toInt().coerceIn(0, 16255)

            return byteArrayOf((quantized / 127).toByte(), (quantized % 127).toByte())
        }

        private fun translateGainLevel(coarse: Byte, fine: Byte): Float {
            return (coarse.toFloat() * 127.0f + fine.toFloat()) / 117.8f - 128.0f
        }

        private fun buildNprnPayload(msb: Byte, lsb: Byte, value: ByteArray, out: ByteArray, offset: Int) {
            MidiNrpnParameter(
                channel = 0x00.toByte(),
                msb = msb,
                lsb = lsb,
                coarse = value[0],
                fine = value[1],
            ).buildInto(out, offset)
        }
    }

    override fun translateChannelLevelValue(level: Float): ByteArray {
        return quantizeGainLevel(level)
    }

    override fun translateRemoteChannelLevelValue(value: ByteArray): Float {
        return translateGainLevel(value[0], value[1])
    }

    override fun translateChannelPanValue(pan: Float): ByteArray {
        val value = ((pan + 1.0f) / 2.0f * 16255.0f).toInt().coerceIn(0, 16255)

        return byteArrayOf((value / 127).toByte(), (value % 127).toByte())
    }

    override fun translateRemoteChannelPanValue(value: ByteArray): Float {
        return (value[0] * 127 + value[1]).toFloat() / 8128.0f - 1.0f
    }

    override fun translateChannelReverbValue(reverb: Float): ByteArray {
        return quantizeGainLevel(reverb)
    }

    override fun translateRemoteChannelReverbValue(value: ByteArray): Float {
        return translateGainLevel(value[0], value[1])
    }

    override fun translateChannelMuteValue(mute: Boolean): ByteArray {
        return if (mute) {
            byteArrayOf(0x00, 0x01)
        } else {
            byteArrayOf(0x00, 0x00)
        }
    }

    override fun translateRemoteChannelMuteValue(value: ByteArray): Boolean {
        return value[1] > 0
    }

    override fun translateGlobalMasterLevelValue(level: Float): ByteArray {
        return quantizeGainLevel(level)
    }

    override fun translateRemoteGlobalMasterLevelValue(value: ByteArray): Float {
        return translateGainLevel(value[0], value[1])
    }

    override fun translateGlobalMonitorLevelValue(level: Float): ByteArray {
        return quantizeGainLevel(level)
    }

    override fun translateRemoteGlobalMonitorLevelValue(value: ByteArray): Float {
        return translateGainLevel(value[0], value[1])
    }

    override fun initializeState(
        config: MixerConfigurations,
        initialStates: List<ControlValue>
    ): List<ByteArray> {
        val states = mutableListOf(
            byteArrayOf(
                0xb0.toByte(), 0x00, 0x00, 0xc0.toByte(), 0x00
            )
        )
        states.addAll(initialStates.map {
            val payload = ByteArray(getMidiPayloadSizeHint(config, it))
            buildMidiPayload(config, it, payload, 0)
            payload
        })

        return states
    }

    override fun parseMidiPayload(
        config: MixerConfigurations,
        packet: ByteArray,
        offset: Int,
        size: Int
    ): ControlValue? {
        val nprn = MidiNrpnParameter.parse(packet, offset, size) ?: return null

        return when (nprn.msb) {
            0x40.toByte() -> {
                // Main LR Level
                val channel = config.channelsByIndex[nprn.lsb.toInt()] ?: return null
                ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.LEVEL,
                    value = byteArrayOf(nprn.coarse, nprn.fine ?: 0)
                )
            }
            0x50.toByte() -> {
                // Pan
                val channel = config.channelsByIndex[nprn.lsb.toInt()] ?: return null
                ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.PAN,
                    value = byteArrayOf(nprn.coarse, nprn.fine ?: 0)
                )
            }
            0x00.toByte() -> {
                // Mute
                val channel = config.channelsByIndex[nprn.lsb.toInt()] ?: return null
                ControlValue.ChannelValue(
                    channelId = channel.id,
                    control = ChannelControlParameter.MUTE,
                    value = byteArrayOf(nprn.coarse, nprn.fine ?: 0)
                )
            }
            0x4c.toByte(), 0x4d.toByte() -> {
                // FX1 send (let's assume it's reverb)
                val index = nprn.msb.toInt() * 127 + nprn.lsb.toInt()
                if (index >= 9672 && ((index - 9672) % 4 == 0)) {
                    val channel = config.channelsByIndex[(index - 9672) / 4] ?: return null
                    ControlValue.ChannelValue(
                        channelId = channel.id,
                        control = ChannelControlParameter.REVERB,
                        value = byteArrayOf(nprn.coarse, nprn.fine ?: 0)
                    )
                } else {
                    null
                }
            }
            else -> null
        }
    }

    override fun buildMidiPayload(
        config: MixerConfigurations,
        value: ControlValue,
        output: ByteArray,
        offset: Int
    ): Int {
        return when (value) {
            is ControlValue.ChannelValue -> {
                val channel = config.channelsById[value.channelId] ?: return 0

                when (value.control) {
                    ChannelControlParameter.LEVEL -> {
                        buildNprnPayload(0x40.toByte(), channel.index.toByte(), value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                    ChannelControlParameter.PAN -> {
                        buildNprnPayload(0x50.toByte(), channel.index.toByte(), value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                    ChannelControlParameter.MUTE -> {
                        buildNprnPayload(0x00.toByte(), channel.index.toByte(), value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                    ChannelControlParameter.REVERB -> {
                        // TODO: make reverb send target configurable
                        val fx1Base = 0x4c * 127 + 0x14
                        val target = fx1Base + channel.index * 4
                        val msb = (target / 127).toByte()
                        val lsb = (target % 127).toByte()
                        buildNprnPayload(msb, lsb, value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                    else -> 0
                }
            }
            is ControlValue.GlobalValue -> {
                when (value.control) {
                    GlobalControlParameter.MASTER_LEVEL -> {
                        buildNprnPayload(0x4f.toByte(), 0x00.toByte(), value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                    GlobalControlParameter.MONITOR_LEVEL -> {
                        buildNprnPayload(0x4f.toByte(), 0x11.toByte(), value.value, output, offset)
                        MidiNrpnParameter.LENGTH_FINE
                    }
                }
            }
        }
    }

    override fun getMidiPayloadSizeHint(
        config: MixerConfigurations,
        value: ControlValue
    ): Int {
        return MidiNrpnParameter.LENGTH_FINE
    }

    override fun flowControlMilliseconds(): Long = 0

    override fun maxPayloadInBatch(): Int = 10

}