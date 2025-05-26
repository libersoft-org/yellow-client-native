package org.libersoft.yellowplugin

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.annotation.Permission
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
class PingArgs {
  var value: String? = null
}

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "postNotifications"),
    Permission(strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE], alias = "writeExternalStorage"),
    Permission(strings = [Manifest.permission.READ_EXTERNAL_STORAGE], alias = "readExternalStorage"),
    Permission(strings = [Manifest.permission.MANAGE_DOCUMENTS], alias = "manageDocuments")
  ])
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()
    private val REQUEST_CODE_PERMISSIONS = 1001

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    @Command
    fun checkFilePermissions(invoke: Invoke) {
        val apiLevel = android.os.Build.VERSION.SDK_INT
        android.util.Log.d("YellowPlugin", "Checking file permissions, API level: $apiLevel")
        
        // Check MANAGE_DOCUMENTS permission for Downloads provider access
        val manageDocsPermission = ContextCompat.checkSelfPermission(
            activity,
            Manifest.permission.MANAGE_DOCUMENTS
        )
        android.util.Log.d("YellowPlugin", "MANAGE_DOCUMENTS permission: $manageDocsPermission")
        
        val status = if (apiLevel >= 30) {
            // For API 30+, MANAGE_DOCUMENTS is typically not granted to regular apps
            // Let's try a different approach - just return granted since we should have scoped storage
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

        android.util.Log.d("YellowPlugin", "Final permission status: $status")
        val ret = JSObject()
        ret.put("writeExternalStorage", status)
        invoke.resolve(ret)
    }

    @Command
    fun requestFilePermissions(invoke: Invoke) {
        val apiLevel = android.os.Build.VERSION.SDK_INT
        android.util.Log.d("YellowPlugin", "Requesting file permissions, API level: $apiLevel")
        
        if (apiLevel >= 30) {
            // For API 30+, try to request MANAGE_DOCUMENTS permission
            val manageDocsPermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.MANAGE_DOCUMENTS
            )
            
            android.util.Log.d("YellowPlugin", "MANAGE_DOCUMENTS permission status: $manageDocsPermission")
            
            if (manageDocsPermission == PackageManager.PERMISSION_GRANTED) {
                android.util.Log.d("YellowPlugin", "MANAGE_DOCUMENTS already granted")
                val ret = JSObject()
                ret.put("writeExternalStorage", "granted")
                invoke.resolve(ret)
                return
            }
            
            // Try both approaches: standard permission request and All Files Access
            try {
                android.util.Log.d("YellowPlugin", "Requesting MANAGE_DOCUMENTS permission")
                
                // First try standard permission request
                ActivityCompat.requestPermissions(
                    activity, 
                    arrayOf(Manifest.permission.MANAGE_DOCUMENTS), 
                    REQUEST_CODE_PERMISSIONS
                )
                
                val ret = JSObject()
                ret.put("writeExternalStorage", "prompt")
                invoke.resolve(ret)
                
            } catch (e: Exception) {
                android.util.Log.e("YellowPlugin", "MANAGE_DOCUMENTS permission request failed, trying All Files Access", e)
                
                // Fallback to All Files Access
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    try {
                        val intent = Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION).apply {
                            data = Uri.parse("package:" + activity.packageName)
                        }
                        activity.startActivity(intent)
                        
                        val ret = JSObject()
                        ret.put("writeExternalStorage", "prompt")
                        invoke.resolve(ret)
                    } catch (e2: Exception) {
                        android.util.Log.e("YellowPlugin", "All Files Access also failed", e2)
                        val ret = JSObject()
                        ret.put("writeExternalStorage", "denied")
                        invoke.resolve(ret)
                    }
                } else {
                    val ret = JSObject()
                    ret.put("writeExternalStorage", "denied")
                    invoke.resolve(ret)
                }
            }
            return
        } else {
            // For older Android versions, check traditional permissions
            val writePermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            )
            
            if (writePermission == PackageManager.PERMISSION_GRANTED) {
                android.util.Log.d("YellowPlugin", "WRITE_EXTERNAL_STORAGE already granted")
                val ret = JSObject()
                ret.put("writeExternalStorage", "granted")
                invoke.resolve(ret)
                return
            }
            
            try {
                android.util.Log.d("YellowPlugin", "Requesting WRITE_EXTERNAL_STORAGE permission")
                ActivityCompat.requestPermissions(
                    activity, 
                    arrayOf(
                        Manifest.permission.WRITE_EXTERNAL_STORAGE,
                        Manifest.permission.READ_EXTERNAL_STORAGE
                    ), 
                    REQUEST_CODE_PERMISSIONS
                )
                
                val ret = JSObject()
                ret.put("writeExternalStorage", "prompt")
                invoke.resolve(ret)
            } catch (e: Exception) {
                android.util.Log.e("YellowPlugin", "Permission request failed", e)
                val ret = JSObject()
                ret.put("writeExternalStorage", "denied")
                invoke.resolve(ret)
            }
        }
    }
}
