package kr.dream_house.osd.views

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.MixerPreferences
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult
import kr.dream_house.osd.navigate
import kr.dream_house.osd.ui.theme.WhitishYellow

@Composable
fun MainScreen(
    modifier: Modifier = Modifier,
    initialSubroute: String,
    sendOpenDoorRequest: suspend () -> DoorLockOpenResult?,
    activeBooking: Booking?,
    parkingStates: List<ParkingState>,
    mixerPreferences: MixerPreferences?,
    onUpdateMixerPreferences: (MixerPreferences) -> Unit,
) {
    val nestedNavController = rememberNavController()

    val navBackStackEntry by nestedNavController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    Row(modifier = modifier.fillMaxSize().background(WhitishYellow)) {
        Box(modifier = Modifier.weight(0.3f).padding(16.dp)) {
            Card(
                colors = CardDefaults.cardColors(containerColor = Color(0xfffff1d7)),
                elevation = CardDefaults.cardElevation(defaultElevation = 6.dp)
            ) {
                Sidebar(
                    sendOpenDoorRequest = sendOpenDoorRequest,
                    activeBooking = activeBooking,
                    parkingStates = parkingStates,
                    currentRoute = currentRoute,
                    onNavigateToMixer = { nestedNavController.navigate(navigation = Navigation.MainScreen.Mixer, nested = true) },
                    onNavigateToUnitInformation = { nestedNavController.navigate(navigation = Navigation.MainScreen.UnitInformation, nested = true) }
                )
            }
        }
        Box(modifier = Modifier.weight(0.7f)) {
            NavHost(navController = nestedNavController, startDestination = initialSubroute) {
                composable(Navigation.MainScreen.Mixer.route()) {
                    MixerControls(
                        mixerPreferences = mixerPreferences,
                        onUpdateMixerPreferences = onUpdateMixerPreferences,
                        customerId = activeBooking?.customerId
                    )
                }
                composable(Navigation.MainScreen.UnitInformation.route()) {
                    UnitInformation()
                }
            }
        }
    }
}