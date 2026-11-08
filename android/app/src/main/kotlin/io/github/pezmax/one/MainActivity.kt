package io.github.pezmax.one

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import io.github.pezmax.one.ui.PezMaxRoot
import io.github.pezmax.one.ui.theme.PezMaxTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            PezMaxTheme {
                PezMaxRoot()
            }
        }
    }
}
