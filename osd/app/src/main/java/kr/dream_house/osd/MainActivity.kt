package kr.dream_house.osd

import android.Manifest
import android.app.KeyguardManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.admin.DevicePolicyManager
import android.content.ComponentName
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import kr.dream_house.osd.midi.ChannelData
import kr.dream_house.osd.midi.GlobalData
import kr.dream_house.osd.midi.LocalMixerController
import kr.dream_house.osd.midi.MidiDeviceManager
import kr.dream_house.osd.midi.MixerController
import kr.dream_house.osd.midi.createMidiDeviceManager
import kr.dream_house.osd.midi.devices.Vm3100ProMixerDevice
import kr.dream_house.osd.mqtt.MqttService
import kr.dream_house.osd.ui.theme.DXETheme
import kr.dream_house.osd.views.App

class MainActivity : ComponentActivity() {
    companion object {
        private const val TAG = "MainActivity"
    }

    private var midiDeviceManager: MidiDeviceManager? = null

    private fun handleNotificationPrerequisites() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
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

        val devicePolicyManager = getSystemService(DevicePolicyManager::class.java)
        if (!devicePolicyManager.isAdminActive(admin)) {
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN).putExtra(
                DevicePolicyManager.EXTRA_DEVICE_ADMIN, admin)
            startActivity(intent)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        requestDeviceAdministratorPermission()
        handleNotificationPrerequisites()

        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
        setShowWhenLocked(true)
        setTurnScreenOn(true)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        val keyguardManager = getSystemService(KeyguardManager::class.java)
        keyguardManager.requestDismissKeyguard(this, object: KeyguardManager.KeyguardDismissCallback() {})

        midiDeviceManager = createMidiDeviceManager()
        val mixerController = midiDeviceManager?.let {
            MixerController(it, Vm3100ProMixerDevice())
        }

        onBackPressedDispatcher.addCallback(this, object: OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {}
        })

        enableEdgeToEdge()
        setContent {
            CompositionLocalProvider(LocalMixerController provides mixerController) {
                DisposableEffect(mixerController) {
                    mixerController?.attach()

                    onDispose {
                        mixerController?.detach()
                    }
                }

                DXETheme {
                    Scaffold( modifier = Modifier.fillMaxSize() ) { innerPadding ->
                        App(
                            modifier = Modifier.padding(innerPadding)
                        )
                    }
                }
            }
        }

        MqttService.startIfNotRunning(applicationContext)
    }
}
