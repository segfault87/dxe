package kr.dream_house.osd.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import kr.dream_house.osd.entities.MixerPreferences
import kr.dream_house.osd.entities.MixerPresets

const val SCENE_ROWS = 3
const val SCENE_COLUMNS = 3

@Composable
fun LoadMixerPresets(
    modifier: Modifier = Modifier,
    prefs: MixerPreferences,
    onSelectMixerPresets: (MixerPresets) -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = modifier.width(600.dp).shadow(16.dp, shape = MaterialTheme.shapes.medium),
            shape = RoundedCornerShape(16.dp),
            color = Color.White,
        ) {
            Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                Text(modifier = Modifier.fillMaxWidth(), text = "믹서 설정 불러오기", textAlign = TextAlign.Center, style = MaterialTheme.typography.headlineSmall)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                FilledTonalButton(
                    modifier = Modifier.padding(bottom = 16.dp).fillMaxWidth(),
                    onClick = {
                        onSelectMixerPresets(prefs.default)
                        onDismiss()
                    }
                ) {
                    Text("기본값", style = MaterialTheme.typography.bodyLarge)
                }
                Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    for (i in 0 until SCENE_ROWS) {
                        Row(
                            modifier = Modifier.padding(bottom = 8.dp).fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            for (j in 0 until SCENE_COLUMNS) {
                                val index = i * SCENE_ROWS + j
                                FilledTonalButton(
                                    modifier = Modifier.weight(1.0f),
                                    colors = ButtonDefaults.filledTonalButtonColors(containerColor = MaterialTheme.colorScheme.primary),
                                    enabled = prefs.scenes.contains(index.toString()),
                                    onClick = {
                                        prefs.scenes[index.toString()]?.let {
                                            onSelectMixerPresets(it)
                                            onDismiss()
                                        }
                                    }
                                ) {
                                    Text((index + 1).toString(), style = MaterialTheme.typography.bodyLarge)
                                }
                            }
                        }
                    }
                }
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                TextButton(modifier = Modifier.fillMaxWidth(), onClick = onDismiss, colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.tertiary)) {
                    Text("닫기", style = MaterialTheme.typography.headlineSmall)
                }
            }
        }
    }
}

@Composable
fun SaveMixerPresets(
    modifier: Modifier = Modifier,
    presets: MixerPresets,
    prefs: MixerPreferences?,
    onUpdatePreferences: (MixerPreferences) -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = modifier.width(600.dp).shadow(16.dp, shape = MaterialTheme.shapes.medium),
            shape = RoundedCornerShape(16.dp),
            color = Color.White,
        ) {
            Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                Text(modifier = Modifier.fillMaxWidth(), text = "믹서 설정 저장", textAlign = TextAlign.Center, style = MaterialTheme.typography.headlineSmall)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                Text(
                    modifier = Modifier.padding(bottom = 16.dp),
                    text = "안내 : 기본값으로 저장하실 경우 다음에 이용하실 때 자동으로 해당 설정이 적용됩니다.")
                FilledTonalButton(
                    modifier = Modifier.padding(bottom = 16.dp).fillMaxWidth(),
                    onClick = {
                        val newPrefs = prefs?.copy(default = presets) ?: MixerPreferences(default = presets, scenes = emptyMap())
                        onUpdatePreferences(newPrefs)
                        onDismiss()
                    }
                ) {
                    Text(text = "기본값", style = MaterialTheme.typography.bodyLarge)
                }
                Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    for (i in 0 until SCENE_ROWS) {
                        Row(
                            modifier = Modifier.padding(bottom = 8.dp).fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            for (j in 0 until SCENE_COLUMNS) {
                                val index = i * SCENE_ROWS + j
                                FilledTonalButton(
                                    modifier = Modifier.weight(1.0f),
                                    colors = ButtonDefaults.filledTonalButtonColors(containerColor = MaterialTheme.colorScheme.primary),
                                    onClick = {
                                        val scenes = (prefs?.scenes?.entries?.associate { (k, v) -> k to v.copy() }?.toMutableMap() ?: mutableMapOf()).also {
                                            it[(index).toString()] = presets
                                        }
                                        val newPrefs = prefs?.copy(scenes = scenes) ?: MixerPreferences(default = MixerPresets(), scenes = scenes)
                                        onUpdatePreferences(newPrefs)
                                        onDismiss()
                                    }
                                ) {
                                    Text((index + 1).toString(), style = MaterialTheme.typography.bodyLarge)
                                }
                            }
                        }
                    }
                }
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                TextButton(modifier = Modifier.fillMaxWidth(), onClick = onDismiss, colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.tertiary)) {
                    Text("닫기", style = MaterialTheme.typography.headlineSmall)
                }
            }
        }
    }
}