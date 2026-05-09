package kr.dream_house.bastion

import android.app.AlertDialog
import android.app.KeyguardManager
import android.app.admin.DevicePolicyManager
import android.content.ComponentName
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.text.InputType
import android.widget.EditText
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultCallback
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    companion object {
        private val OSD_PACKAGE_NAME = if (BuildConfig.DEBUG) {
            "kr.dream_house.osd.debug"
        } else {
            "kr.dream_house.osd"
        }

        private const val OSD_ACTIVITY_NAME = "kr.dream_house.osd.MainActivity"

        private val OSD_COMPONENT_NAME = ComponentName(OSD_PACKAGE_NAME, OSD_ACTIVITY_NAME)
    }

    private lateinit var devicePolicyManager: DevicePolicyManager

    private lateinit var dxeActivityLauncher: ActivityResultLauncher<Intent>
    private val onDxeActivityResult = ActivityResultCallback<ActivityResult> { result ->
        if (result.resultCode != RESULT_OK) {
            lifecycleScope.launch {
                delay(2000)
                startDxeActivity()
            }
        }
    }

    private val isDeviceOwner = mutableStateOf(false)

    private lateinit var prefs: Prefs

    private fun checkDeviceOwner() {
        val admin = ComponentName(applicationContext, DeviceAdminReceiver::class.java)

        if (devicePolicyManager.isDeviceOwnerApp(packageName)) {
            devicePolicyManager.setLockTaskPackages(admin, arrayOf(OSD_PACKAGE_NAME, packageName))
            isDeviceOwner.value = true
        }
    }

    private fun requestDeviceAdministratorPermission() {
        val admin = ComponentName(applicationContext, DeviceAdminReceiver::class.java)

        if (!devicePolicyManager.isAdminActive(admin)) {
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN).putExtra(
                DevicePolicyManager.EXTRA_DEVICE_ADMIN, admin)
            startActivity(intent)
        }
    }

    private fun dismissKeyguard() {
        val keyguardManager = getSystemService<KeyguardManager>()!!
        keyguardManager.requestDismissKeyguard(this, object: KeyguardManager.KeyguardDismissCallback() {})
    }

    private fun setMasterPasswordIfNeeded() {
        if (prefs.masterPassword != null) {
            return
        }

        val input = EditText(this)
        input.setInputType(InputType.TYPE_TEXT_VARIATION_PASSWORD);

        AlertDialog.Builder(this)
            .setTitle("비밀번호를 입력해 주세요")
            .setView(input)
            .setPositiveButton("확인", { dialog, which ->
                prefs.masterPassword = input.text.toString()
                dialog.dismiss()
            })
            .setNegativeButton("취소", { dialog, which ->
                dialog.dismiss()
            })
            .show()
    }

    private fun checkMasterPassword(next: () -> Unit) {
        if (prefs.masterPassword == null) {
            Toast.makeText(this, "비밀번호를 설정해주세요.", Toast.LENGTH_SHORT).show()
            return
        }

        val input = EditText(this)
        input.setInputType(InputType.TYPE_TEXT_VARIATION_PASSWORD);

        AlertDialog.Builder(this)
            .setTitle("비밀번호를 입력해 주세요")
            .setView(input)
            .setPositiveButton("확인", { dialog, which ->
                if (input.text.toString() != prefs.masterPassword) {
                    Toast.makeText(this, "비밀번호가 틀립니다.", Toast.LENGTH_SHORT).show()
                } else {
                    next()
                }
                dialog.dismiss()
            })
            .setNegativeButton("취소", { dialog, which ->
                dialog.dismiss()
            })
            .show()
    }

    private fun startDxeActivity(destination: String? = null) {
        val intent = Intent().apply {
            component = OSD_COMPONENT_NAME
            putExtra("destination", destination)
        }

        dxeActivityLauncher.launch(intent)
    }

    private fun startSystemSettings() {
        checkMasterPassword {
            startActivity(Intent(Settings.ACTION_SETTINGS))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        prefs = Prefs(this)

        dxeActivityLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult(), onDxeActivityResult)
        devicePolicyManager = getSystemService(DevicePolicyManager::class.java)

        requestDeviceAdministratorPermission()

        enableEdgeToEdge()
        setContent {
            val isDeviceOwner by isDeviceOwner

            val coroutineScope = rememberCoroutineScope()

            DisposableEffect(Unit) {
                val job = coroutineScope.launch {
                    while (true) {
                        checkDeviceOwner()
                        delay(10000)
                    }
                }

                return@DisposableEffect onDispose {
                    job.cancel()
                }
            }

            Scaffold( modifier = Modifier.fillMaxSize() ) { innerPadding ->
                Box(Modifier.fillMaxSize().padding(innerPadding), contentAlignment = Alignment.Center) {
                    if (!isDeviceOwner) {
                        Text("기기 소유자가 아닙니다. ADB에서 기기 소유자로 등록해 주세요.")
                    } else {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                            FilledTonalButton(onClick = this@MainActivity::startDxeActivity) {
                                Text("앱 시작")
                            }
                            FilledTonalButton(onClick = {
                                checkMasterPassword {
                                    startDxeActivity("config")
                                }
                            }) {
                                Text("설정")
                            }
                            FilledTonalButton(onClick = { startSystemSettings() }) {
                                Text("시스템 설정")
                            }
                        }
                    }
                }
            }
        }
    }

    override fun onStart() {
        super.onStart()

        checkDeviceOwner()
        dismissKeyguard()

        setMasterPasswordIfNeeded()
    }

    override fun onResume() {
        super.onResume()

        startLockTask()
    }
}
