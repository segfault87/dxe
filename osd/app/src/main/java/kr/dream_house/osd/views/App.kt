package kr.dream_house.osd.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import kr.dream_house.osd.Navigation

@Composable
fun App(modifier: Modifier, startDestination: Any = Navigation.MainScreen) {
    val navController = rememberNavController()

    NavHost(modifier = modifier, navController = navController, startDestination = startDestination) {
        composable<Navigation.MainScreen> {
            MainScreen(navController)
        }
        composable<Navigation.Config> {
            Config(navController)
        }
    }
}