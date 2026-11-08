package io.github.pezmax.one.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import io.github.takahashirinta.kanesumi.core.insets.bottomOverlayPadding
import io.github.takahashirinta.kanesumi.core.theme.MetroText
import io.github.takahashirinta.kanesumi.structure.MetroAppBar
import io.github.takahashirinta.kanesumi.structure.MetroShell

@Composable
fun PezMaxRoot() {
    MetroShell {
        Column(Modifier.fillMaxSize()) {
            MetroAppBar(title = "拼图满绩")
            LazyColumn(contentPadding = bottomOverlayPadding()) {
                item { MetroText("开工中") }
            }
        }
    }
}
