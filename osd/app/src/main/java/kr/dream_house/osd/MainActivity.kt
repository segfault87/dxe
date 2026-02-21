package kr.dream_house.osd

import android.Manifest
import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.admin.DevicePolicyManager
import android.content.ComponentName
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.os.PowerManager
import android.provider.Settings
import android.util.Log
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import androidx.core.content.getSystemService
import androidx.core.net.toUri
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import androidx.navigation.compose.rememberNavController
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kr.dream_house.osd.midi.LocalMixerController
import kr.dream_house.osd.midi.MidiDeviceManager
import kr.dream_house.osd.midi.MixerController
import kr.dream_house.osd.midi.createMidiDeviceManager
import kr.dream_house.osd.midi.devices.Vm3100ProMixerDevice
import kr.dream_house.osd.mqtt.MqttService
import kr.dream_house.osd.mqtt.TopicSubscriber
import kr.dream_house.osd.mqtt.topics.ControlDevice
import kr.dream_house.osd.ui.theme.DXETheme
import kr.dream_house.osd.views.App

class MainActivity : ComponentActivity() {
    companion object {
        private const val TAG = "MainActivity"
    }

    private lateinit var devicePolicyManager: DevicePolicyManager

    private var midiDeviceManager: MidiDeviceManager? = null

    private var mqttServiceBinder: MqttService.LocalBinder? = null
    private val mqttServiceConnection = object: ServiceConnection {
        override fun onServiceConnected(
            name: ComponentName?,
            service: IBinder?
        ) {
            mqttServiceBinder = service as? MqttService.LocalBinder

            mqttServiceBinder?.subscribeTopics()
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            mqttServiceBinder = null
        }
    }

    private var navigationState by mutableStateOf<String?>(null)

    private fun MqttService.LocalBinder.subscribeTopics() {
        val onDeviceLock = object: TopicSubscriber<ControlDevice> {
            override fun onPayload(
                topic: String,
                payload: ControlDevice
            ) {
                payload.lock?.let { lock ->
                    if (lock) {
                        startLockTask()
                    } else {
                        stopLockTask()
                    }
                }
                payload.navigation?.let { navigation ->
                    navigationState = navigation
                }
                payload.finish?.let {
                    if (it) {
                        setResult(RESULT_OK)
                        finish()
                    }
                }
                payload.triggerCrash?.let {
                    if (it) {
                        throw Exception("Crash triggered from remote")
                    }
                }
            }
        }

        subscribe(onDeviceLock)
    }

    private fun ignoreBatteryOptimization() {
        val pm = getSystemService<PowerManager>()!!
        if (!pm.isIgnoringBatteryOptimizations(packageName)) {
            @SuppressLint("BatteryLife")
            val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                data = "package:$packageName".toUri()
            }
            startActivity(intent)
        }
    }

    private fun handleNotificationPrerequisites() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            val activityResultLauncher = registerForActivityResult(ActivityResultContracts.RequestPermission(), { isGranted ->
                if (!isGranted) {
                    Log.e(TAG, "Should grant permission")
                    finish()
                }
            })
            activityResultLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }

        val channel = NotificationChannel("FOREGROUND_SERVICE_NOTIFIER", "DXE Background Service", NotificationManager.IMPORTANCE_LOW)
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.createNotificationChannel(channel)
    }

    private fun requestDeviceAdministratorPermission() {
        val admin = ComponentName(applicationContext, DeviceAdminReceiver::class.java)

        if (!devicePolicyManager.isAdminActive(admin)) {
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN).putExtra(
                DevicePolicyManager.EXTRA_DEVICE_ADMIN, admin)
            startActivity(intent)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        devicePolicyManager = getSystemService(DevicePolicyManager::class.java)

        ignoreBatteryOptimization()
        requestDeviceAdministratorPermission()
        handleNotificationPrerequisites()

        val startDestination = intent.getStringExtra("destination")
        lifecycleScope.launch {
            if (mqttConfigFlow(this@MainActivity).first() == null) {
                navigationState = Navigation.Config.qualifiedRoute()
            }
        }

        val mqttServiceIntent = Intent(this, MqttService::class.java)
        bindService(mqttServiceIntent, mqttServiceConnection, BIND_AUTO_CREATE)
        startService(mqttServiceIntent)

        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        midiDeviceManager = createMidiDeviceManager()
        val mixerController = midiDeviceManager?.let {
            MixerController(it, Vm3100ProMixerDevice())
        }

        onBackPressedDispatcher.addCallback(this, object: OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {}
        })

        enableEdgeToEdge()
        setContent {
            val navController = rememberNavController()

            LaunchedEffect(navigationState) {
                navigationState?.let {
                    navController.navigate(route = it)

                }
            }

            CompositionLocalProvider(LocalMixerController provides mixerController) {
                DisposableEffect(mixerController) {
                    mixerController?.attach()

                    onDispose {
                        mixerController?.detach()
                    }
                }

                DXETheme {
                    App(
                        modifier = Modifier.fillMaxSize(),
                        navController = navController,
                        startDestination = startDestination,
                    )
                }
            }
        }
    }

    override fun onStart() {
        super.onStart()

        startLockTask()
    }

    override fun onDestroy() {
        super.onDestroy()

        unbindService(mqttServiceConnection)
    }
}
