package kr.dream_house.osd.views

import android.annotation.SuppressLint
import android.webkit.WebView
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import kr.dream_house.osd.BuildConfig

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun UnitInformation() {
    val baseUrl = "${BuildConfig.INFORMATION_URL_BASE}/units/${BuildConfig.UNIT_ID}/"

    AndroidView(
        modifier = Modifier.fillMaxSize(),
        factory = { context ->
            WebView(context).apply {
                settings.javaScriptEnabled = true
                loadUrl(baseUrl)
            }
        }
    )
}
