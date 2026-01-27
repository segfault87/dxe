package kr.dream_house.osd.views

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.BoxWithConstraintsScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PointMode
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import kr.dream_house.osd.midi.ChannelData
import kr.dream_house.osd.midi.MixerCapability
import kr.dream_house.osd.midi.PartialChannelDataUpdate
import kr.dream_house.osd.utils.ThreeBandEq
import kotlin.math.log2
import kotlin.math.pow

val MIN_FREQ_LOG2 = log2(20.0f)
val MAX_FREQ_LOG2 = log2(20000.0f)
val FREQ_SPAN_LOG2 = MAX_FREQ_LOG2 - MIN_FREQ_LOG2

fun gainFraction(
    gain: Float,
    gainRange: ClosedFloatingPointRange<Float>,
): Float {
    return 1.0f - ((gain + -gainRange.start) / (gainRange.endInclusive - gainRange.start))
}

@Composable
fun BoxWithConstraintsScope.EqHandle(
    freq: Float,
    freqRange: ClosedFloatingPointRange<Float>?,
    gain: Float,
    gainRange: ClosedFloatingPointRange<Float>,
    onUpdateValues: (Float, Float?) -> Unit,
    size: Dp = 24.dp,
    color: @Composable () -> Color = { MaterialTheme.colorScheme.secondary },
) {
    val freqFraction = (log2(freq) - MIN_FREQ_LOG2) / FREQ_SPAN_LOG2
    val gainFraction = gainFraction(gain, gainRange)

    val half = size * 0.5f

    var posX by remember { mutableStateOf(freqFraction * maxWidth.value) }
    var posY by remember { mutableStateOf(gainFraction * maxHeight.value) }

    Box(modifier = Modifier
        .offset(x = posX.dp - half, y = posY.dp - half)
        .size(size)
        .shadow(
            elevation = 8.dp,
            shape = CircleShape,
            ambientColor = Color.Gray,
            spotColor = Color.DarkGray,
        )
        .background(color = color(), shape = CircleShape)
        .pointerInput(Unit) {
            detectDragGestures { change, dragAmount ->
                change.consume()

                val newFreq = if (freqRange != null) {
                    val bound = (log2(freqRange.start) - MIN_FREQ_LOG2) / FREQ_SPAN_LOG2 * maxWidth.value .. (log2(freqRange.endInclusive) - MIN_FREQ_LOG2) / FREQ_SPAN_LOG2 * maxWidth.value
                    posX = (posX + dragAmount.x).coerceIn(bound)
                    2.0f.pow(MIN_FREQ_LOG2 + (posX / maxWidth.value) * FREQ_SPAN_LOG2).coerceIn(freqRange)
                } else {
                    null
                }

                posY = (posY + dragAmount.y).coerceIn(0.0f..maxHeight.value)
                val newGain = ((1.0f - (posY / maxHeight.value)) * (gainRange.endInclusive - gainRange.start) + gainRange.start).coerceIn(gainRange)

                onUpdateValues(newGain, newFreq)
            }
        }
    )
    Text(
        modifier = Modifier
            .width(80.dp)
            .offset(x = posX.dp - 40.dp, y = posY.dp + size),
        textAlign = TextAlign.Center,
        text = "${freq.toInt()} Hz",
        style = MaterialTheme.typography.labelMedium)
}

@Composable
fun BoxWithConstraintsScope.ThreeBandEqCurve(
    eq: ThreeBandEq,
    gainRange: ClosedFloatingPointRange<Float>,
) {
    var points by remember { mutableStateOf(mutableListOf<Offset>()) }

    Canvas(modifier = Modifier.fillMaxSize()) {
        val width = maxWidth.roundToPx()
        val height = maxHeight.roundToPx()

        if (points.size != width) {
            points = MutableList(width) { Offset(0.0f, 0.0f) }
        }

        for (i in 0 until width) {
            val freq = 2.0f.pow(MIN_FREQ_LOG2 + (i.toFloat() / width.toFloat()) * FREQ_SPAN_LOG2)
            val y = gainFraction(eq.calculateGain(freq), gainRange)

            points[i] = Offset(i.toFloat(), y * height)
        }

        drawPoints(
            points = points,
            pointMode = PointMode.Polygon,
            color = Color.LightGray,
            strokeWidth = 1.0f,
        )
    }
}

@Composable
fun ThreeBandEqView(
    capabilities: Set<MixerCapability>,
    channelData: ChannelData,
    onUpdateValue: (PartialChannelDataUpdate) -> Unit,
    midQRange: ClosedFloatingPointRange<Float>,
    modifier: Modifier = Modifier,
) {
    val eq = remember { ThreeBandEq(
        channelData.eqLowFreq, channelData.eqLowLevel,
        channelData.eqMidFreq, channelData.eqMidLevel, channelData.eqMidQ,
        channelData.eqHighFreq, channelData.eqHighLevel
    ) }

    var midQ by remember { mutableStateOf(log2(channelData.eqMidQ)) }

    val lowFreqAdjustable = remember { capabilities.contains(MixerCapability.CHANNEL_THREE_BAND_EQ_LOW_FREQ) }
    val midFreqAdjustable = remember { capabilities.contains(MixerCapability.CHANNEL_THREE_BAND_EQ_MID_FREQ) }
    val highFreqAdjustable = remember { capabilities.contains(MixerCapability.CHANNEL_THREE_BAND_EQ_HIGH_FREQ) }

    Column(modifier = modifier.fillMaxWidth()) {
        BoxWithConstraints(modifier = Modifier.fillMaxWidth().height(200.dp)) {
            ThreeBandEqCurve(eq, -12.0f..12.0f)
            EqHandle(
                freq = channelData.eqLowFreq,
                freqRange = if (lowFreqAdjustable) { 20.0f..2000.0f } else { null },
                gain = channelData.eqLowLevel,
                gainRange = -12.0f..12.0f,
                onUpdateValues = { level, freq ->
                    onUpdateValue(PartialChannelDataUpdate(eqLowLevel = level, eqLowFreq = freq))
                    eq.updateLow(freq ?: channelData.eqLowFreq, level)
                },
            )
            EqHandle(
                freq = channelData.eqMidFreq,
                freqRange = if (midFreqAdjustable) { 200.0f..8000.0f } else { null },
                gain = channelData.eqMidLevel,
                gainRange = -12.0f..12.0f,
                onUpdateValues = { level, freq ->
                    onUpdateValue(PartialChannelDataUpdate(eqMidLevel = level, eqMidFreq = freq))
                    eq.updateMid(freq ?: channelData.eqMidFreq, level, channelData.eqMidQ)
                },
            )
            EqHandle(
                freq = channelData.eqHighFreq,
                freqRange = if (highFreqAdjustable) { 400.0f..20000.0f } else { null },
                gain = channelData.eqHighLevel,
                gainRange = -12.0f..12.0f,
                onUpdateValues = { level, freq ->
                    onUpdateValue(PartialChannelDataUpdate(eqHighLevel = level, eqHighFreq = freq))
                    eq.updateHigh(freq ?: channelData.eqHighFreq, level)
                },
            )
        }
        Row(modifier = Modifier.fillMaxWidth()) {
            Spacer(modifier = Modifier.weight(1.0f))
            Slider(
                modifier = Modifier.weight(1.0f),
                valueRange = midQRange,
                value = midQ,
                onValueChange = {
                    midQ = it
                    val qValue = 2.0f.pow(midQ)

                    eq.updateMid(channelData.eqMidFreq, channelData.eqMidLevel, qValue)
                    onUpdateValue(PartialChannelDataUpdate(eqMidQ = qValue))
                }
            )
            Spacer(modifier = Modifier.weight(1.0f))
        }
    }
}

@Composable
fun ThreeBandEqPopup(
    channelName: String,
    capabilities: Set<MixerCapability>,
    channelData: ChannelData,
    onUpdateValue: (PartialChannelDataUpdate) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = modifier.width(800.dp).shadow(16.dp, shape = MaterialTheme.shapes.medium),
            shape = RoundedCornerShape(16.dp),
            color = Color.White,
        ) {
            Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                Text(modifier = Modifier.fillMaxWidth(), text = "$channelName - EQ 설정", textAlign = TextAlign.Center, style = MaterialTheme.typography.headlineSmall)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                ThreeBandEqView(
                    capabilities = capabilities,
                    channelData = channelData,
                    onUpdateValue = onUpdateValue,
                    midQRange = -1.0f..3.0f
                )
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                TextButton(modifier = Modifier.fillMaxWidth(), onClick = onDismiss, colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.tertiary)) {
                    Text("닫기", style = MaterialTheme.typography.headlineSmall)
                }
            }
        }
    }
}