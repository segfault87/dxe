package kr.dream_house.osd.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kr.dream_house.osd.MQTT_DEFAULT_PORT
import kr.dream_house.osd.crashCollectorUrlFlow
import kr.dream_house.osd.mqttConfigFlow
import kr.dream_house.osd.setCrashCollectorUrl
import kr.dream_house.osd.setMqttPrefs

@Composable
fun Config(onDismiss: () -> Unit) {
    val context = LocalContext.current
    val mqttConfig by mqttConfigFlow(context).collectAsState(initial = null)
    val crashCollectorConfig by crashCollectorUrlFlow(context).collectAsState(null)
    val coroutineScope = rememberCoroutineScope()

    var host by remember { mutableStateOf("") }
    var port by remember { mutableStateOf("$MQTT_DEFAULT_PORT") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var crashCollectorUrl by remember { mutableStateOf("") }

    LaunchedEffect(mqttConfig) {
        mqttConfig?.let {
            host = it.host
            port = it.port.toString()
            username = it.username ?: ""
            password = it.password ?: ""
        }
    }

    LaunchedEffect(crashCollectorConfig) {
        crashCollectorConfig?.let {
            crashCollectorUrl = it
        }
    }

    Column(modifier = Modifier.padding(24.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Host: ")
            TextField(value = host, onValueChange = { host = it })
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Port: ")
            TextField(
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Number,
                ),
                value = port,
                onValueChange = {
                    port = it.filter { it.isDigit() }
                }
            )
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Username: ")
            TextField(value = username, onValueChange = { username = it })
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Password: ")
            TextField(
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Password,
                ),
                visualTransformation = PasswordVisualTransformation(),
                value = password,
                onValueChange = { password = it }
            )
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Crash Collector URL: ")
            TextField(value = crashCollectorUrl, onValueChange = { crashCollectorUrl = it })
        }

        FilledTonalButton(
            enabled = host.isNotEmpty(),
            onClick = { coroutineScope.launch {
                setMqttPrefs(context, host, port.toInt(), username.ifEmpty { null }, password.ifEmpty { null })
                setCrashCollectorUrl(context, crashCollectorUrl.ifEmpty { null })
                onDismiss()
            } }
        ) {
            Text("Save")
        }

    }
}