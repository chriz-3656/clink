package com.chriz.clink.ui.components

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.chriz.clink.data.model.ClinkMode

@Composable
fun ModeSegmentedControl(
    selectedMode: ClinkMode,
    onModeSelected: (ClinkMode) -> Unit,
    modifier: Modifier = Modifier
) {
    Surface(
        shape = RoundedCornerShape(24.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
        tonalElevation = 4.dp,
        modifier = modifier
    ) {
        SingleChoiceSegmentedButtonRow(
            modifier = Modifier.padding(4.dp)
        ) {
            SegmentedButton(
                shape = SegmentedButtonDefaults.itemShape(index = 0, count = 2),
                selected = selectedMode == ClinkMode.ABSOLUTE_ANNOTATION,
                onClick = { onModeSelected(ClinkMode.ABSOLUTE_ANNOTATION) }
            ) {
                Text("Annotation")
            }
            SegmentedButton(
                shape = SegmentedButtonDefaults.itemShape(index = 1, count = 2),
                selected = selectedMode == ClinkMode.RELATIVE_TABLET,
                onClick = { onModeSelected(ClinkMode.RELATIVE_TABLET) }
            ) {
                Text("Tablet")
            }
        }
    }
}
