package kr.dream_house.osd

import android.Manifest
import android.app.ActivityOptions
import android.app.KeyguardManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.admin.DevicePolicyManager
import android.content.ComponentName
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.os.PowerManager
import android.provider.Settings
import android.util.Log
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class DpcActivity : ComponentActivity() {
    companion object {
        private const val TAG = "DpcActivity"
    }

    private val isDeviceOwner = mutableStateOf(false)

    private lateinit var devicePolicyManager: DevicePolicyManager

    private fun ignoreBatteryOptimization() {
        val pm = getSystemService<PowerManager>()!!
        if (!pm.isIgnoringBatteryOptimizations(packageName)) {
            val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                data = Uri.parse("package:$packageName")
            }
            startActivity(intent)
        }
    }

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

        if (!devicePolicyManager.isAdminActive(admin)) {
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN).putExtra(
                DevicePolicyManager.EXTRA_DEVICE_ADMIN, admin)
            startActivity(intent)
        }
    }

    private fun checkDeviceOwner() {
        val admin = ComponentName(applicationContext, DeviceAdminReceiver::class.java)

        if (devicePolicyManager.isDeviceOwnerApp(packageName)) {
            devicePolicyManager.setLockTaskPackages(admin, arrayOf(packageName))
            isDeviceOwner.value = true
        }
    }

    private fun startDxeActivity(destination: String? = null) {
        stopLockTask()

        val options = ActivityOptions.makeBasic()
        options.setLockTaskEnabled(true)

        val launchIntent = Intent(this, MainActivity::class.java).apply {
            destination?.let {
                putExtra("destination", it)
            }
        }
        startActivity(launchIntent, options.toBundle())
    }

    override fun onStart() {
        super.onStart()

        checkDeviceOwner()
        if (isDeviceOwner.value) {
            lifecycleScope.launch {
                delay(2000)
                startDxeActivity()
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val keyguardManager = getSystemService<KeyguardManager>()!!
        keyguardManager.requestDismissKeyguard(this, object: KeyguardManager.KeyguardDismissCallback() {})

        devicePolicyManager = getSystemService(DevicePolicyManager::class.java)

        ignoreBatteryOptimization()
        requestDeviceAdministratorPermission()
        handleNotificationPrerequisites()

        setContent {
            val isDeviceOwner by isDeviceOwner

            val coroutineScope = rememberCoroutineScope()

            LaunchedEffect(Unit) {
                coroutineScope.launch {
                    while (true) {
                        checkDeviceOwner()
                        delay(60000)
                    }
                }
            }

            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                if (!isDeviceOwner) {
                    Text("기기 소유자가 아닙니다. ADB에서 기기 소유자로 등록해 주세요.")
                } else {
                    Column(verticalArrangement = Arrangement.spacedBy(16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                        FilledTonalButton(onClick = this@DpcActivity::startDxeActivity) {
                            Text("앱 시작")
                        }
                        FilledTonalButton(onClick = { startDxeActivity("config") }) {
                            Text("설정")
                        }
                    }
                }
            }
        }
    }
}