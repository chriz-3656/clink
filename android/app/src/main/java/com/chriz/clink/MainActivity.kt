package com.chriz.clink

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.animation.Crossfade
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import com.chriz.clink.ui.screens.*
import com.chriz.clink.ui.theme.ClinkTheme
import com.chriz.clink.viewmodel.MainViewModel

enum class Screen {
    HOME,
    DRAWING,
    SETTINGS,
    SESSIONS
}

class MainActivity : ComponentActivity() {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            ClinkTheme(darkTheme = viewModel.preferences.darkTheme) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    var currentScreen by remember { mutableStateOf(Screen.HOME) }

                    Crossfade(targetState = currentScreen, label = "ScreenTransition") { screen ->
                        when (screen) {
                            Screen.HOME -> HomeScreen(
                                viewModel = viewModel,
                                onNavigateDrawing = { currentScreen = Screen.DRAWING },
                                onNavigateSettings = { currentScreen = Screen.SETTINGS },
                                onNavigateSessions = { currentScreen = Screen.SESSIONS }
                            )
                            Screen.DRAWING -> DrawingScreen(
                                viewModel = viewModel,
                                onNavigateBack = { currentScreen = Screen.HOME }
                            )
                            Screen.SETTINGS -> SettingsScreen(
                                viewModel = viewModel,
                                onNavigateBack = { currentScreen = Screen.HOME }
                            )
                            Screen.SESSIONS -> SessionsScreen(
                                viewModel = viewModel,
                                onNavigateBack = { currentScreen = Screen.HOME },
                                onSessionLoaded = { currentScreen = Screen.DRAWING }
                            )
                        }
                    }
                }
            }
        }
    }
}
