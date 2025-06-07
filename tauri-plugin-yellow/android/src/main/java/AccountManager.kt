package org.libersoft.yellowplugin

import android.content.Context
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import kotlinx.coroutines.*

class AccountManager(
    private val context: Context,
    private val messageHandler: (String, JSONObject) -> Unit
) {
    companion object {
        private const val TAG = "AccountManager"
    }
    
    private val accounts = mutableMapOf<String, Account>()
    private val encryptedStorage = EncryptedStorage(context)
    private val coroutineScope = CoroutineScope(Dispatchers.Main + SupervisorJob())
    
    init {
        // Load and initialize accounts on creation
        loadAccounts()
    }
    
    fun updateAccountsConfig(configJson: String) {
        Log.d(TAG, "Updating accounts configuration")
        
        coroutineScope.launch {
            try {
                // Save to encrypted storage
                if (encryptedStorage.saveAccountsConfig(configJson)) {
                    // Parse and update accounts
                    val accountsArray = JSONArray(configJson)
                    updateAccountsFromConfig(accountsArray)
                    
                    // Notify service to update notification
                    notifyServiceAccountsUpdated()
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to update accounts config", e)
            }
        }
    }
    
    private fun loadAccounts() {
        coroutineScope.launch {
            try {
                val configJson = encryptedStorage.getAccountsConfig()
                if (configJson != null) {
                    val accountsArray = JSONArray(configJson)
                    updateAccountsFromConfig(accountsArray)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load accounts", e)
            }
        }
    }
    
    private suspend fun updateAccountsFromConfig(accountsArray: JSONArray) = withContext(Dispatchers.IO) {
        val configuredAccountIds = mutableSetOf<String>()
        
        // Process each account in the config
        for (i in 0 until accountsArray.length()) {
            val accountConfig = accountsArray.getJSONObject(i)
            val accountId = accountConfig.getString("id")
            configuredAccountIds.add(accountId)
            
            val credentials = parseCredentials(accountConfig.getJSONObject("credentials"))
            val settings = parseSettings(accountConfig.getJSONObject("settings"))
            val enabled = accountConfig.getBoolean("enabled")
            
            // Check if account exists
            val existingAccount = accounts[accountId]
            
            if (existingAccount != null) {
                // For simplicity, destroy and recreate if credentials changed
                val existingCreds = existingAccount.credentials
                if (existingCreds.server != credentials.server ||
                    existingCreds.address != credentials.address ||
                    existingCreds.password != credentials.password ||
                    existingCreds.retryNonce != credentials.retryNonce) {
                    
                    Log.d(TAG, "Credentials changed for account $accountId, recreating...")
                    destroyAccount(accountId)
                    createAccount(accountId, credentials, settings, enabled)
                } else {
                    // Just update enabled state if needed
                    if (enabled && !existingAccount.isEnabled) {
                        existingAccount.enable()
                    } else if (!enabled && existingAccount.isEnabled) {
                        existingAccount.disable()
                    }
                }
            } else {
                // Create new account
                createAccount(accountId, credentials, settings, enabled)
            }
        }
        
        // Remove accounts that are no longer in config
        val accountsToRemove = accounts.keys.filter { it !in configuredAccountIds }
        accountsToRemove.forEach { accountId ->
            Log.d(TAG, "Removing account $accountId (not in config)")
            destroyAccount(accountId)
        }
    }
    
    private fun createAccount(
        accountId: String,
        credentials: Account.AccountCredentials,
        settings: Account.AccountSettings,
        enabled: Boolean
    ) {
        Log.d(TAG, "Creating account: $accountId")
        
        val account = Account(context, accountId, credentials, settings, messageHandler)
        accounts[accountId] = account
        
        if (enabled) {
            account.enable()
        }
    }
    
    private fun destroyAccount(accountId: String) {
        Log.d(TAG, "Destroying account: $accountId")
        
        accounts[accountId]?.let { account ->
            account.destroy()
            accounts.remove(accountId)
        }
    }
    
    private fun parseCredentials(credentialsJson: JSONObject): Account.AccountCredentials {
        return Account.AccountCredentials(
            server = credentialsJson.getString("server"),
            address = credentialsJson.getString("address"),
            password = credentialsJson.getString("password"),
            retryNonce = credentialsJson.optInt("retry_nonce", 0)
        )
    }
    
    private fun parseSettings(settingsJson: JSONObject): Account.AccountSettings {
        val modulesEnabled = mutableListOf<String>()
        
        // Parse modules_enabled array if present
        if (settingsJson.has("modules_enabled")) {
            val modulesArray = settingsJson.getJSONArray("modules_enabled")
            for (i in 0 until modulesArray.length()) {
                modulesEnabled.add(modulesArray.getString(i))
            }
        }
        
        return Account.AccountSettings(
            displayName = settingsJson.optString("display_name", null),
            modulesEnabled = modulesEnabled
        )
    }
    
    private fun notifyServiceAccountsUpdated() {
        try {
            val intent = android.content.Intent(context, YellowForegroundService::class.java).apply {
                action = YellowForegroundService.ACTION_UPDATE_ACCOUNTS
            }
            context.startService(intent)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to notify service about account update", e)
        }
    }
    
    fun getActiveAccountsCount(): Int {
        return accounts.count { it.value.isEnabled }
    }
    
    fun sendToAccount(accountId: String, message: JSONObject): Boolean {
        val account = accounts[accountId]
        return if (account != null && account.isEnabled) {
            account.sendMessage(message)
            true
        } else {
            Log.e(TAG, "Account not found or disabled: $accountId")
            false
        }
    }
    
    fun destroy() {
        // Destroy all accounts
        accounts.values.forEach { it.destroy() }
        accounts.clear()
        
        // Cancel coroutine scope
        coroutineScope.cancel()
    }
}