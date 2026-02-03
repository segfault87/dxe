package kr.dream_house.osd.mqtt.topics

import kotlinx.serialization.Serializable
import kr.dream_house.osd.BuildConfig
import kr.dream_house.osd.IdentityId
import kr.dream_house.osd.entities.MixerPreferences
import kr.dream_house.osd.mqtt.TopicSpec

@Serializable
data class SetMixerPreferences(
    val prefs: MixerPreferences,
) {
    @Serializable
    data class Update(
        val customerId: IdentityId,
        val prefs: MixerPreferences,
    ) {
        companion object {
            const val TOPIC_NAME = "dxe/mixer_preferences/${BuildConfig.UNIT_ID}/set"
        }
    }

    companion object : TopicSpec {
        override val topicName = "dxe/mixer_preferences/${BuildConfig.UNIT_ID}"
    }
}