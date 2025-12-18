package kr.dream_house.osd.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.navigation.NavController
import kotlinx.coroutines.launch
import kr.dream_house.osd.MQTT_DEFAULT_PORT
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.mqttConfigFlow
import kr.dream_house.osd.setMqttPrefs

@Composable
fun Config(navController: NavController) {
    val context = LocalContext.current
    val mqttConfig by mqttConfigFlow(context).collectAsState(initial = null)
    val coroutineScope = rememberCoroutineScope()

    var host by remember { mutableStateOf("") }
    var port by remember { mutableStateOf("$MQTT_DEFAULT_PORT") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }

    LaunchedEffect(mqttConfig) {
        mqttConfig?.let {
            host = it.host
            port = it.port.toString()
            username = it.username ?: ""
            password = it.password ?: ""
        }
    }

    Column {
        Row {
            Text("Host: ")
            TextField(value = host, onValueChange = { host = it })
        }

        Row {
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

        Row {
            Text("Username: ")
            TextField(value = username, onValueChange = { username = it })
        }

        Row {
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

        Button(
            enabled = host.isNotEmpty(),
            onClick = { coroutineScope.launch {
                setMqttPrefs(context, host, port.toInt(), username.ifEmpty { null }, password.ifEmpty { null })
                navController.navigate(route = Navigation.MainScreen)
            } }
        ) {
            Text("Save")
        }

    }
}