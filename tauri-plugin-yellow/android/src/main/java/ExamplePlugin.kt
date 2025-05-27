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
import android.content.ContentValues
import android.provider.MediaStore
import java.io.File
import android.os.Environment
import android.media.MediaScannerConnection
import java.io.FileOutputStream

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
    Permission(strings = [Manifest.permission.READ_EXTERNAL_STORAGE], alias = ALIAS_READ_EXTERNAL_STORAGE)
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
        val args = invoke.getArgs()
        val filePath = args.getString("filePath")
        val fileName = args.getString("fileName") ?: "download"
        val mimeType = args.getString("mimeType") ?: "application/octet-stream"
        
        if (filePath == null) {
            invoke.reject("No file path provided")
            return
        }
        
        try {
            val sourceFile = File(activity.filesDir, filePath)
            if (!sourceFile.exists()) {
                invoke.reject("Source file not found: $filePath")
                return
            }
            
            val apiLevel = android.os.Build.VERSION.SDK_INT
            
            if (apiLevel >= 29) {
                // Use MediaStore for Android Q and above
                val resolver = activity.contentResolver
                val contentValues = ContentValues().apply {
                    put(MediaStore.Downloads.DISPLAY_NAME, fileName)
                    put(MediaStore.Downloads.MIME_TYPE, mimeType)
                    put(MediaStore.Downloads.IS_PENDING, 1)
                }
                
                val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
                if (uri != null) {
                    // Stream file directly without loading into memory
                    resolver.openOutputStream(uri)?.use { outputStream ->
                        sourceFile.inputStream().use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    
                    contentValues.clear()
                    contentValues.put(MediaStore.Downloads.IS_PENDING, 0)
                    resolver.update(uri, contentValues, null, null)
                    
                    val ret = JSObject()
                    ret.put("success", true)
                    ret.put("uri", uri.toString())
                    ret.put("path", "Downloads/$fileName")
                    invoke.resolve(ret)
                } else {
                    invoke.reject("Failed to create file in Downloads")
                }
            } else {
                // For older Android versions
                val downloadsDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
                val destFile = File(downloadsDir, fileName)
                
                // Stream copy
                sourceFile.inputStream().use { input ->
                    destFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                
                // Notify media scanner
                MediaScannerConnection.scanFile(
                    activity,
                    arrayOf(destFile.absolutePath),
                    arrayOf(mimeType),
                    null
                )
                
                val ret = JSObject()
                ret.put("success", true)
                ret.put("path", destFile.absolutePath)
                invoke.resolve(ret)
            }
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to export file", e)
            invoke.reject("Failed to export file: ${e.message}")
        }
    }
    
    @Command
    fun saveToDownloads(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName") ?: "download"
        val mimeType = args.getString("mimeType") ?: "application/octet-stream"
        val dataBase64 = args.getString("data")
        
        if (dataBase64 == null) {
            invoke.reject("No data provided")
            return
        }
        
        try {
            val data = android.util.Base64.decode(dataBase64, android.util.Base64.DEFAULT)
            val apiLevel = android.os.Build.VERSION.SDK_INT
            
            if (apiLevel >= 29) {
                // Use MediaStore for Android Q and above
                val resolver = activity.contentResolver
                val contentValues = ContentValues().apply {
                    put(MediaStore.Downloads.DISPLAY_NAME, fileName)
                    put(MediaStore.Downloads.MIME_TYPE, mimeType)
                    put(MediaStore.Downloads.IS_PENDING, 1)
                }
                
                val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
                if (uri != null) {
                    resolver.openOutputStream(uri)?.use { outputStream ->
                        outputStream.write(data)
                    }
                    
                    contentValues.clear()
                    contentValues.put(MediaStore.Downloads.IS_PENDING, 0)
                    resolver.update(uri, contentValues, null, null)
                    
                    val ret = JSObject()
                    ret.put("success", true)
                    ret.put("uri", uri.toString())
                    ret.put("path", "Downloads/$fileName")
                    invoke.resolve(ret)
                } else {
                    invoke.reject("Failed to create file in Downloads")
                }
            } else {
                // For older Android versions, use direct file access
                val downloadsDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
                if (!downloadsDir.exists()) {
                    downloadsDir.mkdirs()
                }
                
                val file = File(downloadsDir, fileName)
                FileOutputStream(file).use { outputStream ->
                    outputStream.write(data)
                }
                
                val ret = JSObject()
                ret.put("success", true)
                ret.put("path", file.absolutePath)
                invoke.resolve(ret)
            }
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to save file", e)
            invoke.reject("Failed to save file: ${e.message}")
        }
    }
}
