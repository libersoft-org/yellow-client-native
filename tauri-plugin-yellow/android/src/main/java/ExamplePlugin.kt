package org.libersoft.yellowplugin

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.activity.result.ActivityResult
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.annotation.Permission
import app.tauri.annotation.ActivityCallback
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import androidx.core.app.ActivityCompat.OnRequestPermissionsResultCallback

private const val ALIAS_WRITE_EXTERNAL_STORAGE: String = "writeExternalStorage"
private const val ALIAS_READ_EXTERNAL_STORAGE: String = "readExternalStorage"

@InvokeArg
class PingArgs {
  var value: String? = null
}

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "postNotifications"),
    Permission(strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE], alias = ALIAS_WRITE_EXTERNAL_STORAGE),
    Permission(strings = [Manifest.permission.READ_EXTERNAL_STORAGE], alias = ALIAS_READ_EXTERNAL_STORAGE),
    Permission(strings = [Manifest.permission.FOREGROUND_SERVICE], alias = "foregroundService")
  ])
class ExamplePlugin(private val activity: Activity): Plugin(activity), OnRequestPermissionsResultCallback {
    private val implementation = Example()
    private val files = Files(activity)
    private val encryptedStorage = EncryptedStorage(activity)
    private val REQUEST_CODE_PERMISSIONS = 1001
    private val REQUEST_CODE_NOTIFICATIONS = 1004  // Separate code for notifications
    private val CREATE_FILE_REQUEST_CODE = 1002
    private val OPEN_FILE_REQUEST_CODE = 1003
    
    private var pendingInvoke: Invoke? = null
    private var pendingFileName: String? = null
    private var pendingMimeType: String? = null
    
    init {
        android.util.Log.d("YellowPlugin", "ExamplePlugin initialized - API level: ${Build.VERSION.SDK_INT}")
        // Always try to start service - let the method handle permission checks
        startServiceOnAppStartup()
    }
    
    
    private fun startServiceOnAppStartup() {
        try {
            // Check if service is already running
            if (YellowForegroundService.isRunning()) {
                android.util.Log.d("YellowPlugin", "Foreground service already running, skipping startup")
                return
            }
            
            // For Android 13+, check notification permission first
            if (Build.VERSION.SDK_INT >= 33) {
                val notificationPermission = ContextCompat.checkSelfPermission(
                    activity,
                    Manifest.permission.POST_NOTIFICATIONS
                )
                android.util.Log.d("YellowPlugin", "Notification permission status: $notificationPermission")
                
                if (notificationPermission != PackageManager.PERMISSION_GRANTED) {
                    android.util.Log.d("YellowPlugin", "Requesting POST_NOTIFICATIONS permission")
                    ActivityCompat.requestPermissions(
                        activity,
                        arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                        REQUEST_CODE_NOTIFICATIONS
                    )
                    return
                }
            }
            
            android.util.Log.d("YellowPlugin", "Starting foreground service on app startup")
            
            val serviceIntent = Intent(activity, YellowForegroundService::class.java).apply {
                action = YellowForegroundService.ACTION_START
                putExtra(YellowForegroundService.EXTRA_TITLE, "Yellow Service")
                putExtra(YellowForegroundService.EXTRA_MESSAGE, "Service is running")
            }
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                activity.startForegroundService(serviceIntent)
            } else {
                activity.startService(serviceIntent)
            }
            
            android.util.Log.d("YellowPlugin", "Foreground service started on app startup")
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to start service on app startup", e)
        }
    }

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    @Command
    override fun checkPermissions(invoke: Invoke) {
        val apiLevel = android.os.Build.VERSION.SDK_INT
        android.util.Log.d("YellowPlugin", "Checking permissions, API level: $apiLevel")
        
        val writeStatus = if (apiLevel >= 30) {
            // For API 30+, use scoped storage approach
            android.util.Log.d("YellowPlugin", "API 30+, using scoped storage approach")
            "granted"
        } else {
            // For older Android versions, check traditional permissions
            val writePermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            )
            android.util.Log.d("YellowPlugin", "WRITE_EXTERNAL_STORAGE permission: $writePermission")
            when (writePermission) {
                PackageManager.PERMISSION_GRANTED -> "granted"
                else -> "denied"
            }
        }
        
        val readStatus = if (apiLevel >= 30) {
            "granted"
        } else {
            val readPermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.READ_EXTERNAL_STORAGE
            )
            when (readPermission) {
                PackageManager.PERMISSION_GRANTED -> "granted"
                else -> "denied"
            }
        }

        android.util.Log.d("YellowPlugin", "Write permission status: $writeStatus, Read permission status: $readStatus")
        val ret = JSObject()
        ret.put("writeExternalStorage", writeStatus)
        ret.put("readExternalStorage", readStatus)
        invoke.resolve(ret)
    }

    @Command
    override fun requestPermissions(invoke: Invoke) {
        val apiLevel = android.os.Build.VERSION.SDK_INT
        android.util.Log.d("YellowPlugin", "Requesting permissions, API level: $apiLevel")
        
        // Parse the permissions parameter
        val args = invoke.getArgs()
        val permissionsArray = args.optJSONArray("permissions")
        val requestedPermissions = mutableListOf<String>()
        
        if (permissionsArray != null) {
            for (i in 0 until permissionsArray.length()) {
                val permission = permissionsArray.getString(i)
                when (permission) {
                    "writeExternalStorage" -> requestedPermissions.add("writeExternalStorage")
                    "readExternalStorage" -> requestedPermissions.add("readExternalStorage")
                }
            }
        } else {
            // If no specific permissions requested, request both
            requestedPermissions.add("writeExternalStorage")
            requestedPermissions.add("readExternalStorage")
        }
        
        if (apiLevel >= 30) {
            // For API 30+, we'll handle storage permissions differently
            android.util.Log.d("YellowPlugin", "API 30+, using scoped storage - returning granted")
            val ret = JSObject()
            ret.put("writeExternalStorage", "granted")
            ret.put("readExternalStorage", "granted")
            invoke.resolve(ret)
            return
        } else {
            // For older Android versions, check and request traditional permissions
            val permissionsToRequest = mutableListOf<String>()
            var allGranted = true
            
            if (requestedPermissions.contains("writeExternalStorage")) {
                val writePermission = ContextCompat.checkSelfPermission(
                    activity,
                    Manifest.permission.WRITE_EXTERNAL_STORAGE
                )
                if (writePermission != PackageManager.PERMISSION_GRANTED) {
                    permissionsToRequest.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
                    allGranted = false
                }
            }
            
            if (requestedPermissions.contains("readExternalStorage")) {
                val readPermission = ContextCompat.checkSelfPermission(
                    activity,
                    Manifest.permission.READ_EXTERNAL_STORAGE
                )
                if (readPermission != PackageManager.PERMISSION_GRANTED) {
                    permissionsToRequest.add(Manifest.permission.READ_EXTERNAL_STORAGE)
                    allGranted = false
                }
            }
            
            if (allGranted) {
                android.util.Log.d("YellowPlugin", "All requested permissions already granted")
                val ret = JSObject()
                ret.put("writeExternalStorage", "granted")
                ret.put("readExternalStorage", "granted")
                invoke.resolve(ret)
                return
            }
            
            try {
                android.util.Log.d("YellowPlugin", "Requesting permissions: ${permissionsToRequest.joinToString()}")
                ActivityCompat.requestPermissions(
                    activity, 
                    permissionsToRequest.toTypedArray(), 
                    REQUEST_CODE_PERMISSIONS
                )
                
                // Return prompt status while permission dialog is shown
                val ret = JSObject()
                ret.put("writeExternalStorage", "prompt")
                ret.put("readExternalStorage", "prompt")
                invoke.resolve(ret)
            } catch (e: Exception) {
                android.util.Log.e("YellowPlugin", "Permission request failed", e)
                val ret = JSObject()
                ret.put("writeExternalStorage", "denied")
                ret.put("readExternalStorage", "denied")
                invoke.resolve(ret)
            }
        }
    }
    
    @Command
    fun exportFileToDownloads(invoke: Invoke) {
        files.exportFileToDownloads(invoke)
    }
    
    @Command
    fun saveToDownloads(invoke: Invoke) {
        files.saveToDownloads(invoke)
    }
    
    @Command
    fun createFile(invoke: Invoke) {
        files.createFile(invoke)
    }
    
    @Command
    fun appendToFile(invoke: Invoke) {
        files.appendToFile(invoke)
    }
    
    @Command
    fun renameFile(invoke: Invoke) {
        files.renameFile(invoke)
    }
    
    @Command
    fun deleteFile(invoke: Invoke) {
        files.deleteFile(invoke)
    }
    
    @Command
    fun fileExists(invoke: Invoke) {
        files.fileExists(invoke)
    }
    
    @Command
    fun getFileSize(invoke: Invoke) {
        files.getFileSize(invoke)
    }
    
    @Command
    fun openSaveDialog(invoke: Invoke) {
        files.openSaveDialog(
            invoke,
            { pendingInv ->
                pendingInvoke = pendingInv
                val args = pendingInv.getArgs()
                pendingFileName = args.getString("fileName") ?: "download"
                pendingMimeType = args.getString("mimeType") ?: "application/octet-stream"
            },
            { inv, intent, callback -> startActivityForResult(inv, intent, callback) }
        )
    }
    
    @Command
    fun saveFileToUri(invoke: Invoke) {
        files.saveFileToUri(invoke)
    }
    
    @Command
    fun saveAccountsConfig(invoke: Invoke) {
        try {
            val args = invoke.getArgs()
            val configJson = args.getString("configJson")
            
            if (configJson == null) {
                android.util.Log.e("YellowPlugin", "saveAccountsConfig called with null configJson")
                invoke.reject("Missing configJson parameter")
                return
            }
            
            android.util.Log.d("YellowPlugin", "Saving accounts config, length: ${configJson.length}")
            val success = encryptedStorage.saveAccountsConfig(configJson)
            
            val ret = JSObject()
            ret.put("success", success)
            if (success) {
                android.util.Log.d("YellowPlugin", "Accounts config saved successfully")
                invoke.resolve(ret)
            } else {
                android.util.Log.e("YellowPlugin", "Failed to save accounts config")
                invoke.reject("Failed to save accounts config")
            }
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Exception in saveAccountsConfig", e)
            invoke.reject("Failed to save accounts config: ${e.message}")
        }
    }
    
    @ActivityCallback
    fun handleSaveDialogResult(invoke: Invoke, result: androidx.activity.result.ActivityResult) {
        val resultCode = result.resultCode
        val data = result.data
        android.util.Log.d("YellowPlugin", "handleSaveDialogResult called - resultCode: $resultCode, hasData: ${data != null}")
        
        if (resultCode == Activity.RESULT_OK && data?.data != null) {
            val uri = data.data!!
            android.util.Log.d("YellowPlugin", "Save dialog success, URI: $uri")
            val ret = JSObject()
            ret.put("success", true)
            ret.put("uri", uri.toString())
            ret.put("fileName", pendingFileName)
            ret.put("mimeType", pendingMimeType)
            invoke.resolve(ret)
        } else {
            android.util.Log.d("YellowPlugin", "Save dialog cancelled or failed - resultCode: $resultCode")
            invoke.reject("Save dialog cancelled")
        }
        
        pendingInvoke = null
        pendingFileName = null
        pendingMimeType = null
    }
    
    @Command
    fun startForegroundService(invoke: Invoke) {
        val args = invoke.getArgs()
        val title = args.getString("title") ?: "Yellow Service"
        val message = args.getString("message") ?: "Service is running"
        
        try {
            android.util.Log.d("YellowPlugin", "Starting foreground service - title: $title, message: $message")
            
            val serviceIntent = Intent(activity, YellowForegroundService::class.java).apply {
                action = YellowForegroundService.ACTION_START
                putExtra(YellowForegroundService.EXTRA_TITLE, title)
                putExtra(YellowForegroundService.EXTRA_MESSAGE, message)
            }
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                activity.startForegroundService(serviceIntent)
            } else {
                activity.startService(serviceIntent)
            }
            
            val ret = JSObject()
            ret.put("success", true)
            invoke.resolve(ret)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to start foreground service", e)
            invoke.reject("Failed to start foreground service: ${e.message}")
        }
    }
    
    @Command
    fun stopForegroundService(invoke: Invoke) {
        try {
            android.util.Log.d("YellowPlugin", "Stopping foreground service")
            
            val serviceIntent = Intent(activity, YellowForegroundService::class.java).apply {
                action = YellowForegroundService.ACTION_STOP
            }
            
            activity.startService(serviceIntent)
            
            val ret = JSObject()
            ret.put("success", true)
            invoke.resolve(ret)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to stop foreground service", e)
            invoke.reject("Failed to stop foreground service: ${e.message}")
        }
    }
    
    @Command
    fun checkNotificationPermissionAndStartService(invoke: Invoke) {
        // Check notification permission and start service if granted
        val hasPermission = if (Build.VERSION.SDK_INT >= 33) {
            ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
        } else {
            true // Not needed for Android < 13
        }
        
        if (hasPermission && !YellowForegroundService.isRunning()) {
            startServiceOnAppStartup()
        }
        
        val ret = JSObject()
        ret.put("hasPermission", hasPermission)
        ret.put("serviceRunning", YellowForegroundService.isRunning())
        invoke.resolve(ret)
    }
    
    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        android.util.Log.d("YellowPlugin", "onRequestPermissionsResult: requestCode=$requestCode, permissions=${permissions.contentToString()}, results=${grantResults.contentToString()}")
        
        if (requestCode == REQUEST_CODE_NOTIFICATIONS) {
            val notificationPermissionIndex = permissions.indexOf(Manifest.permission.POST_NOTIFICATIONS)
            if (notificationPermissionIndex >= 0 && grantResults[notificationPermissionIndex] == PackageManager.PERMISSION_GRANTED) {
                android.util.Log.d("YellowPlugin", "POST_NOTIFICATIONS permission granted, starting service")
                startServiceOnAppStartup()
            } else {
                android.util.Log.w("YellowPlugin", "POST_NOTIFICATIONS permission denied")
            }
        }
    }
}
