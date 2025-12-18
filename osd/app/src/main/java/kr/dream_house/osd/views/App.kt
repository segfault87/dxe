package kr.dream_house.osd.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import kr.dream_house.osd.Navigation

@Composable
fun App(modifier: Modifier) {
    val navController = rememberNavController()

    NavHost(navController = navController, startDestination = Navigation.MainScreen) {
        composable<Navigation.MainScreen> {
            MainScreen(navController)
        }
        composable<Navigation.Config> {
            Config(navController)
        }
    }
}