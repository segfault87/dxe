package kr.dream_house.osd.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kr.dream_house.osd.Navigation
import kr.dream_house.osd.entities.Booking
import kr.dream_house.osd.entities.ParkingState
import kr.dream_house.osd.mqtt.topics.DoorLockOpenResult

@Composable
fun Sidebar(
    modifier: Modifier = Modifier,
    sendOpenDoorRequest: suspend () -> DoorLockOpenResult?,
    activeBooking: Booking?,
    parkingStates: List<ParkingState>,
    currentRoute: String?,
    onNavigateToMixer: () -> Unit,
    onNavigateToUnitInformation: () -> Unit
) {
    Column(modifier = modifier) {
        Column(modifier = Modifier.fillMaxSize()) {
            RealTimeInformation(
                sendOpenDoorRequest = sendOpenDoorRequest,
                activeBooking = activeBooking,
                parkingStates = parkingStates
            )

            ElevatedButton(
                modifier = Modifier.fillMaxWidth().padding(8.dp),
                colors = ButtonDefaults.elevatedButtonColors(
                    contentColor = MaterialTheme.colorScheme.tertiary
                ),
                onClick = onNavigateToMixer,
                enabled = currentRoute != Navigation.MainScreen.Mixer.route(),
            ) {
                Text(
                    modifier = Modifier.padding(16.dp),
                    text = "믹서 설정",
                    style = MaterialTheme.typography.titleLarge
                )
            }
            ElevatedButton(
                modifier = Modifier.fillMaxWidth().padding(8.dp),
                colors = ButtonDefaults.elevatedButtonColors(
                    contentColor = MaterialTheme.colorScheme.tertiary
                ),
                onClick = onNavigateToUnitInformation,
                enabled = currentRoute != Navigation.MainScreen.UnitInformation.route(),
            ) {
                Text(
                    modifier = Modifier.padding(16.dp),
                    text = "도움말",
                    style = MaterialTheme.typography.titleLarge
                )
            }
        }
    }
}