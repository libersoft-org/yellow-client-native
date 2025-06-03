package org.libersoft.yellowplugin

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

class EncryptedStorage(private val context: Context) {
    companion object {
        private const val TAG = "EncryptedStorage"
        private const val PREFS_NAME = "yellow_encrypted_prefs"
        private const val ACCOUNTS_CONFIG_KEY = "accounts_config"
    }
    
    private val masterKey: MasterKey by lazy {
        MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
    }
    
    private val encryptedPrefs: SharedPreferences by lazy {
        EncryptedSharedPreferences.create(
            context,
            PREFS_NAME,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }
    
    fun saveAccountsConfig(configJson: String): Boolean {
        return try {
            Log.d(TAG, "Saving accounts config to encrypted storage")
            encryptedPrefs.edit()
                .putString(ACCOUNTS_CONFIG_KEY, configJson)
                .apply()
            Log.d(TAG, "Accounts config saved successfully")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to save accounts config", e)
            false
        }
    }
    
    fun getAccountsConfig(): String? {
        return try {
            val config = encryptedPrefs.getString(ACCOUNTS_CONFIG_KEY, null)
            Log.d(TAG, "Retrieved accounts config from encrypted storage")
            config
        } catch (e: Exception) {
            Log.e(TAG, "Failed to retrieve accounts config", e)
            null
        }
    }
    
    fun clearAccountsConfig(): Boolean {
        return try {
            encryptedPrefs.edit()
                .remove(ACCOUNTS_CONFIG_KEY)
                .apply()
            Log.d(TAG, "Accounts config cleared from encrypted storage")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to clear accounts config", e)
            false
        }
    }
}