package kr.dream_house.bastion

import android.content.Context

class Prefs(context: Context) {

    companion object {
        private val PREFS_ID = "dxe_bastion_prefs"

        private val KEY_MASTER_PASSWORD = "master_password"
    }

    private val sharedPrefs = context.getSharedPreferences(PREFS_ID, Context.MODE_PRIVATE)

    var masterPassword: String?
        get() = sharedPrefs.getString(KEY_MASTER_PASSWORD, null)
        set(value) = sharedPrefs.edit().putString(KEY_MASTER_PASSWORD, value).apply()

}