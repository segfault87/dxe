package kr.dream_house.osd

import android.app.KeyguardManager
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.core.content.getSystemService
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import kr.dream_house.osd.midi.LocalMixerController
import kr.dream_house.osd.midi.MidiDeviceManager
import kr.dream_house.osd.midi.MixerController
import kr.dream_house.osd.midi.createMidiDeviceManager
import kr.dream_house.osd.midi.devices.Vm3100ProMixerDevice
import kr.dream_house.osd.mqtt.MqttService
import kr.dream_house.osd.ui.theme.DXETheme
import kr.dream_house.osd.views.App

class MainActivity : ComponentActivity() {
    private var midiDeviceManager: MidiDeviceManager? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val startDestination = when (intent.getStringExtra("destination")) {
            null, "main" -> Navigation.MainScreen
            "config" -> Navigation.Config
            else -> throw Exception("Invalid navigation key")
        }

        android.util.Log.d("XXX", "des: $startDestination")

        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        val keyguardManager = getSystemService<KeyguardManager>()!!
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
                            modifier = Modifier.padding(innerPadding),
                            startDestination = startDestination,
                        )
                    }
                }
            }
        }

        MqttService.startIfNotRunning(applicationContext)
    }

    override fun onResume() {
        super.onResume()

        startLockTask()
    }
}
