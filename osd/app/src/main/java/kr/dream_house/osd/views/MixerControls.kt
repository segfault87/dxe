package kr.dream_house.osd.views

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kr.dream_house.osd.IdentityId
import kr.dream_house.osd.entities.MixerPreferences
import kr.dream_house.osd.entities.MixerPresets
import kr.dream_house.osd.entities.PartialChannelDataUpdate
import kr.dream_house.osd.entities.PartialGlobalDataUpdate
import kr.dream_house.osd.midi.ChannelData
import kr.dream_house.osd.midi.GlobalData
import kr.dream_house.osd.midi.LocalMixerController
import kr.dream_house.osd.midi.MixerCapability
import kr.dream_house.osd.midi.updateFrom
import kr.dream_house.osd.views.unit_default.TroubleshootingContact

@Composable
private fun MixerRow(
    name: String,
    capabilities: Set<MixerCapability>,
    channelData: ChannelData,
    onChangeLevel: (Float) -> Unit,
    onChangePan: (Float) -> Unit,
    onChangeReverb: (Float) -> Unit,
    onChangeMute: (Boolean) -> Unit,
    onOpenEqPopup: () -> Unit,
) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
        Text(modifier = Modifier.width(200.dp), text = name)
        Slider(
            modifier = Modifier.weight(1.0f),
            enabled = capabilities.contains(MixerCapability.CHANNEL_LEVEL),
            value = channelData.level,
            valueRange = 0f..1f,
            onValueChange = onChangeLevel,
            colors = SliderDefaults.colors(
                thumbColor = MaterialTheme.colorScheme.tertiary,
                activeTrackColor = MaterialTheme.colorScheme.tertiary
            )
        )
        Slider(
            modifier = Modifier.width(100.dp),
            enabled = capabilities.contains(MixerCapability.CHANNEL_REVERB),
            value = channelData.reverb,
            valueRange = 0f..1f,
            onValueChange = onChangeReverb
        )
        CenteredSlider(
            modifier = Modifier.width(100.dp),
            enabled = capabilities.contains(MixerCapability.CHANNEL_PAN),
            value = channelData.pan,
            centerThreshold = 0.1f,
            valueRange = -1f..1f,
            onValueChanged = onChangePan,
        )
        FilledTonalButton(
            modifier = Modifier.width(80.dp),
            enabled = capabilities.contains(MixerCapability.CHANNEL_THREE_BAND_EQ_LEVEL),
            onClick = onOpenEqPopup,
        ) {
            Text("설정")
        }
        Switch(
            checked = channelData.mute,
            enabled = capabilities.contains(MixerCapability.CHANNEL_MUTE),
            onCheckedChange = {
                onChangeMute(it)
            }
        )
    }
}

@Composable
private fun GlobalControlRow(
    capability: Set<MixerCapability>,
    globalData: GlobalData,
    onChangeMasterLevel: (Float) -> Unit,
    onChangeMonitorLevel: (Float) -> Unit,
) {
    Row(modifier = Modifier.padding(bottom = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(modifier = Modifier.padding(end = 8.dp), text = "마스터 음량")
        Slider(
            modifier = Modifier.weight(1.0f),
            enabled = capability.contains(MixerCapability.GLOBAL_MASTER_LEVEL),
            value = globalData.masterLevel,
            valueRange = 0f..1f,
            onValueChange = onChangeMasterLevel,
            colors = SliderDefaults.colors(
                thumbColor = MaterialTheme.colorScheme.secondary,
                activeTrackColor = MaterialTheme.colorScheme.secondary
            )
        )
        Text(modifier = Modifier.padding(start = 24.dp, end = 8.dp), text = "모니터 음량")
        Slider(
            modifier = Modifier.width(300.dp),
            enabled = capability.contains(MixerCapability.GLOBAL_MONITOR_LEVEL),
            value = globalData.monitorLevel,
            valueRange = 0f..1f,
            onValueChange = onChangeMonitorLevel,
            colors = SliderDefaults.colors(
                thumbColor = MaterialTheme.colorScheme.secondary,
                activeTrackColor = MaterialTheme.colorScheme.secondary
            )
        )
    }
}

@Composable
fun MixerControls(mixerPreferences: MixerPreferences?, onUpdateMixerPreferences: (MixerPreferences) -> Unit, customerId: IdentityId?) {
    if (LocalMixerController.current == null) {
        TroubleshootingContact(message = "믹서가 연결되어 있지 않습니다. 위 연락처로 문의해주시기 바랍니다.")
        return
    }

    val mixerController = LocalMixerController.current!!
    val capabilities = remember { mixerController.capabilities }
    val coroutineScope = rememberCoroutineScope()

    val state by mixerController.state.collectAsState()
    val isMixerConnected by mixerController.isConnected.collectAsState()

    var eqPopup by remember { mutableStateOf<Int?>(null) }
    var showLoadPreset by remember { mutableStateOf<MixerPreferences?>(null) }
    var showSavePreset by remember { mutableStateOf<MixerPresets?>(null) }

    val scrollState = rememberScrollState()

    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
            Row(
                modifier = Modifier.padding(bottom = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Text(modifier = Modifier.width(200.dp), text = "채널명", fontWeight = FontWeight.Bold)
                Text(
                    modifier = Modifier.weight(1.0f),
                    text = "음량",
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    modifier = Modifier.width(100.dp),
                    text = "리버브",
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    modifier = Modifier.width(100.dp),
                    text = "밸런스",
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    modifier = Modifier.width(80.dp),
                    text = "EQ",
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    modifier = Modifier.width(50.dp),
                    text = "뮤트",
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.Bold
                )
            }
            Column(
                modifier = Modifier.weight(1.0f).verticalScroll(scrollState),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                mixerController.channels.mapIndexed { idx, name ->
                    val data = state.channels[idx]
                    MixerRow(
                        name = name,
                        capabilities = capabilities,
                        channelData = data,
                        onChangeLevel = {
                            mixerController.updateValues(idx, PartialChannelDataUpdate(level = it))
                        },
                        onChangePan = {
                            mixerController.updateValues(idx, PartialChannelDataUpdate(pan = it))
                        },
                        onChangeReverb = {
                            mixerController.updateValues(idx, PartialChannelDataUpdate(reverb = it))
                        },
                        onChangeMute = {
                            mixerController.updateValues(idx, PartialChannelDataUpdate(mute = it))
                        },
                        onOpenEqPopup = {
                            eqPopup = idx
                        },
                    )
                }
            }
            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))
            GlobalControlRow(
                capabilities,
                globalData = state.globals,
                onChangeMasterLevel = {
                    mixerController.updateValues(PartialGlobalDataUpdate(masterLevel = it))
                },
                onChangeMonitorLevel = {
                    mixerController.updateValues(PartialGlobalDataUpdate(monitorLevel = it))
                }
            )
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Spacer(modifier = Modifier.weight(1.0f))
                FilledTonalButton(
                    onClick = {
                        mixerPreferences?.let {
                            showLoadPreset = it
                        }
                    },
                    enabled = mixerPreferences != null,
                ) {
                    Text("믹서 설정 불러오기", style = MaterialTheme.typography.bodyLarge)
                }
                FilledTonalButton(
                    onClick = {
                        showSavePreset = mixerController.snapshot()
                    },
                    enabled = customerId != null,
                ) {
                    Text("믹서 설정 저장", style = MaterialTheme.typography.bodyLarge)
                }
            }
        }

        eqPopup?.let { channel ->
            ThreeBandEqPopup(
                channelName = mixerController.channels[channel],
                capabilities = capabilities,
                channelData = state.channels[channel],
                onUpdateValue = {
                    mixerController.updateValues(channel, it)
                },
                onDismiss = {
                    eqPopup = null
                }
            )
        }

        showLoadPreset?.let { prefs ->
            LoadMixerPresets(
                prefs = prefs,
                onSelectMixerPresets = { presets ->
                    coroutineScope.launch {
                        mixerController.updateInitialChannelStates(
                            presets.channels.map { ChannelData().updateFrom(it) },
                            GlobalData().updateFrom(presets.globals)
                        )
                    }
                },
                onDismiss = {
                    showLoadPreset = null
                }
            )
        }

        showSavePreset?.let { presets ->
            SaveMixerPresets(
                presets = presets,
                prefs = mixerPreferences,
                onUpdatePreferences = onUpdateMixerPreferences,
                onDismiss = {
                    showSavePreset = null
                }
            )
        }

        if (!isMixerConnected) {
            TroubleshootingContact(
                modifier = Modifier.background(Color(0xccffffff)),
                message = "믹서가 연결되어 있지 않습니다. 위 연락처로 문의해주시기 바랍니다.")
        }
    }
}