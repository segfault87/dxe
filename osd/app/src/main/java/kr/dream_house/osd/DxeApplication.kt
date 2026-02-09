package kr.dream_house.osd

import android.app.Application
import android.content.Context
import org.acra.data.StringFormat
import org.acra.ktx.initAcra

class DxeApplication : Application() {
    override fun attachBaseContext(base: Context?) {
        super.attachBaseContext(base)

        initAcra {
            buildConfigClass = BuildConfig::class.java
            reportFormat = StringFormat.JSON
        }
    }
}