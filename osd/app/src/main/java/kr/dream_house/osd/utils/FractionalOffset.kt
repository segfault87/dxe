package kr.dream_house.osd.utils

import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.layout
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

fun Modifier.fractionalOffset(x: Float, y: Float, xOffset: Dp = 0.dp, yOffset: Dp = 0.dp) = this.then(
    Modifier.layout { measurable, constraints ->
        val placeable = measurable.measure(constraints)
        layout(placeable.width, placeable.height) {
            placeable.placeRelative(
                x = (constraints.maxWidth * x + xOffset.toPx()).toInt(),
                y = (constraints.maxHeight * y + yOffset.toPx()).toInt()
            )
        }
    }
)
