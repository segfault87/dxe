package kr.dream_house.osd

import android.content.Context
import android.util.Log
import com.google.auto.service.AutoService
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.acra.config.CoreConfiguration
import org.acra.data.CrashReportData
import org.acra.sender.ReportSender
import org.acra.sender.ReportSenderFactory
import java.net.HttpURLConnection
import java.net.URL

class CrashCollector(private val crashCollectorUrlFlow: Flow<String?>) : ReportSender {
    companion object {
        private const val TAG = "CrashCollector"
    }

    @OptIn(DelicateCoroutinesApi::class, ExperimentalCoroutinesApi::class)
    override fun send(context: Context, errorContent: CrashReportData) {
        val url = runBlocking {
            crashCollectorUrlFlow.first()
        }
        if (url == null) {
            Log.i(TAG, "Crash collector URL is empty. skipping...")
        }

        val json = errorContent.toJSON().toByteArray(Charsets.UTF_8)

        val connection = URL(url).openConnection() as HttpURLConnection
        connection.apply {
            requestMethod = "POST"
            setRequestProperty("Content-Type", "application/json; utf-8")
            setRequestProperty("Accept", "application/json");
            doOutput = true
        }

        try {
            val outputStream = connection.getOutputStream()
            outputStream.write(json, 0, json.size)

            val inputStream = connection.getInputStream()
            val buf = ByteArray(1000)
            while (inputStream.read(buf) > 0) {
            }
        } catch (e: Exception) {
            Log.e(TAG, "Could not send crash report: $e")
            return
        }

        Log.i(TAG, "Crash report sent successfully")
    }
}

@AutoService(ReportSenderFactory::class)
class CrashCollectorFactory : ReportSenderFactory {
    override fun create(
        context: Context,
        config: CoreConfiguration
    ): ReportSender {
        return CrashCollector(crashCollectorUrlFlow(context))
    }
}
