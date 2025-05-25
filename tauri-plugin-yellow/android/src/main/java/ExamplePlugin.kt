package org.libersoft.yellowplugin

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
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
    Permission(strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE], alias = "writeExternalStorage"),
    Permission(strings = [Manifest.permission.READ_EXTERNAL_STORAGE], alias = "readExternalStorage")
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
        // On Android 11+ (API 30+), WRITE_EXTERNAL_STORAGE is deprecated
        // Apps have scoped storage access by default
        val apiLevel = android.os.Build.VERSION.SDK_INT
        
        val status = if (apiLevel >= 30) {
            // For API 30+, we have scoped storage access by default
            "granted"
        } else {
            // For older Android versions, check the permission
            val writePermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            )
            when (writePermission) {
                PackageManager.PERMISSION_GRANTED -> "granted"
                else -> "denied"
            }
        }

        val ret = JSObject()
        ret.put("writeExternalStorage", status)
        invoke.resolve(ret)
    }

    @Command
    fun requestFilePermissions(invoke: Invoke) {
        val apiLevel = android.os.Build.VERSION.SDK_INT
        
        if (apiLevel >= 30) {
            // For API 30+, we have scoped storage access by default
            // No permission request needed for app-specific directories
            val ret = JSObject()
            ret.put("writeExternalStorage", "granted")
            invoke.resolve(ret)
            return
        }

        // For older Android versions, handle traditional permissions
        val writePermission = ContextCompat.checkSelfPermission(
            activity,
            Manifest.permission.WRITE_EXTERNAL_STORAGE
        )

        if (writePermission == PackageManager.PERMISSION_GRANTED) {
            val ret = JSObject()
            ret.put("writeExternalStorage", "granted")
            invoke.resolve(ret)
            return
        }

        // Check if we should show rationale
        if (ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.WRITE_EXTERNAL_STORAGE)) {
            // Show rationale to user
            val ret = JSObject()
            ret.put("writeExternalStorage", "denied")
            invoke.resolve(ret)
            return
        }

        // Request permissions for older Android versions
        val permissions = arrayOf(
            Manifest.permission.WRITE_EXTERNAL_STORAGE,
            Manifest.permission.READ_EXTERNAL_STORAGE
        )

        try {
            ActivityCompat.requestPermissions(activity, permissions, REQUEST_CODE_PERMISSIONS)
            
            // Return "prompt" to indicate we've requested permissions
            val ret = JSObject()
            ret.put("writeExternalStorage", "prompt")
            invoke.resolve(ret)
        } catch (e: Exception) {
            // If permission request fails, return denied
            val ret = JSObject()
            ret.put("writeExternalStorage", "denied")
            invoke.resolve(ret)
        }
    }
}
