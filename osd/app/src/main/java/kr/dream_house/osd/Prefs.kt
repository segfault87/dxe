package kr.dream_house.osd

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

private const val KEY_MQTT_HOST = "MQTT_HOST"
private const val KEY_MQTT_PORT = "MQTT_PORT"
private const val KEY_MQTT_USERNAME = "MQTT_USERNAME"
private const val KEY_MQTT_PASSWORD = "MQTT_PASSWORD"

const val MQTT_DEFAULT_PORT = 1883

data class MqttConfig(
    val host: String,
    val port: Int,
    val username: String?,
    val password: String?,
)

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "prefs")

suspend fun setMqttPrefs(context: Context, host: String, port: Int, username: String?, password: String?) {
    context.dataStore.edit { prefs ->
        prefs[stringPreferencesKey(KEY_MQTT_HOST)] = host
        prefs[intPreferencesKey(KEY_MQTT_PORT)] = port
        if (username != null) {
            prefs[stringPreferencesKey(KEY_MQTT_USERNAME)] = username
        } else {
            prefs.remove(stringPreferencesKey(KEY_MQTT_USERNAME))
        }
        if (password != null) {
            prefs[stringPreferencesKey(KEY_MQTT_PASSWORD)] = password
        } else {
            prefs.remove(stringPreferencesKey(KEY_MQTT_PASSWORD))
        }

    }
}

fun mqttConfigFlow(context: Context): Flow<MqttConfig?> {
    return context.dataStore.data.map { prefs ->
        val host = prefs[stringPreferencesKey(KEY_MQTT_HOST)]
        val port = prefs[intPreferencesKey(KEY_MQTT_PORT)] ?: MQTT_DEFAULT_PORT
        val username = prefs[stringPreferencesKey(KEY_MQTT_USERNAME)]
        val password = prefs[stringPreferencesKey(KEY_MQTT_PASSWORD)]

        if (host != null) {
            MqttConfig(host, port, username, password)
        } else {
            null
        }
    }
}