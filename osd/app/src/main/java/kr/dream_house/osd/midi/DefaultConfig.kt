package kr.dream_house.osd.midi

// TODO: make it dynamically configurable
object DefaultConfig {
    val MIXER_CONFIGURATIONS = mutableMapOf<String, MixerConfigurations>(
        "roland_vm3100pro" to MixerConfigurations(
            channels = listOf(
                MixerChannelConfig(
                    id = "wm1",
                    name = "무선마이크 1",
                    index = 2,
                ),
                MixerChannelConfig(
                    id = "wm2",
                    name = "무선마이크 2",
                    index = 3,
                ),
                MixerChannelConfig(
                    id = "kb",
                    name = "건반 (채널 11/12)",
                    stereo = true,
                    index = 10,
                ),
                MixerChannelConfig(
                    id = "aux",
                    name = "채널 9/10",
                    stereo = true,
                    index = 8,
                ),
                MixerChannelConfig(
                    id = "mic1",
                    name = "채널 1",
                    index = 0,
                ),
                MixerChannelConfig(
                    id = "mic2",
                    name = "채널 2",
                    index = 1,
                ),
                MixerChannelConfig(
                    id = "line3",
                    name = "채널 3",
                    index = 2,
                ),
                MixerChannelConfig(
                    id = "line4",
                    name = "채널 4",
                    index = 3,
                ),
                MixerChannelConfig(
                    id = "line5",
                    name = "채널 5",
                    index = 4,
                ),
                MixerChannelConfig(
                    id = "line6",
                    name = "채널 6",
                    index = 5,
                ),
                MixerChannelConfig(
                    id = "line7",
                    name = "채널 7",
                    index = 6,
                ),
                MixerChannelConfig(
                    id = "line8",
                    name = "채널 8",
                    index = 7,
                ),
            ),
        ),
        "ah_sq" to MixerConfigurations(
            channels = listOf(
                MixerChannelConfig(
                    id = "wm1",
                    name = "무선마이크 1",
                    index = 0,
                ),
                MixerChannelConfig(
                    id = "wm2",
                    name = "무선마이크 2",
                    index = 1,
                ),
                MixerChannelConfig(
                    id = "kbd_modx",
                    name = "키보드 (MODX8+)",
                    capabilityReverb = false,
                    stereo = true,
                    index = 12,
                ),
                MixerChannelConfig(
                    id = "kbd_nord",
                    name = "키보드 (NORD)",
                    capabilityReverb = false,
                    stereo = true,
                    index = 14,
                ),
                MixerChannelConfig(
                    id = "aux",
                    name = "휴대폰",
                    stereo = true,
                    capabilityReverb = false,
                    index = 44,
                ),
                MixerChannelConfig(
                    id = "mic1",
                    name = "마이크 1",
                    index = 2,
                ),
                MixerChannelConfig(
                    id = "mic2",
                    name = "마이크 2",
                    index = 3,
                ),
                MixerChannelConfig(
                    id = "mic3",
                    name = "마이크 3",
                    index = 4,
                ),
                MixerChannelConfig(
                    id = "di1",
                    name = "DI 1",
                    index = 5,
                ),
                MixerChannelConfig(
                    id = "di2",
                    name = "DI 2",
                    index = 6,
                ),
                MixerChannelConfig(
                    id = "line1",
                    name = "라인 1",
                    index = 7,
                ),
                MixerChannelConfig(
                    id = "line2",
                    name = "라인 2",
                    index = 8,
                ),
                MixerChannelConfig(
                    id = "line3",
                    name = "라인 3",
                    stereo = true,
                    index = 40,
                ),
                MixerChannelConfig(
                    id = "line4",
                    name = "라인 4",
                    stereo = true,
                    index = 42,
                ),
                MixerChannelConfig(
                    id = "mtr_m",
                    name = "MTR (모노)",
                    capabilityReverb = false,
                    index = 9,
                ),
                MixerChannelConfig(
                    id = "mtr_s",
                    name = "MTR (스테레오)",
                    capabilityReverb = false,
                    stereo = true,
                    index = 10,
                ),
            ),
        ),
    )
}