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
    private val CREATE_FILE_REQUEST_CODE = 1002
    private val OPEN_FILE_REQUEST_CODE = 1003
    
    private var pendingInvoke: Invoke? = null
    private var pendingFileName: String? = null
    private var pendingMimeType: String? = null

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
    
    @Command
    fun createFile(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName") ?: "file"
        val content = args.getString("content") ?: ""
        
        try {
            val file = File(activity.filesDir, fileName)
            file.writeText(content)
            android.util.Log.d("YellowPlugin", "Created file: ${file.absolutePath}, size: ${file.length()} bytes")
            
            val ret = JSObject()
            ret.put("success", true)
            ret.put("path", fileName)
            invoke.resolve(ret)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to create file", e)
            invoke.reject("Failed to create file: ${e.message}")
        }
    }
    
    @Command
    fun appendToFile(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName")
        val dataBase64 = args.getString("data")
        
        if (fileName == null || dataBase64 == null) {
            invoke.reject("Missing fileName or data")
            return
        }
        
        try {
            val file = File(activity.filesDir, fileName)
            val data = android.util.Base64.decode(dataBase64, android.util.Base64.DEFAULT)
            
            file.appendBytes(data)
            android.util.Log.d("YellowPlugin", "Appended data to file: ${file.absolutePath}, new size: ${file.length()} bytes")

            val ret = JSObject()
            ret.put("success", true)
            invoke.resolve(ret)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to append to file", e)
            invoke.reject("Failed to append to file: ${e.message}")
        }
    }
    
    @Command
    fun renameFile(invoke: Invoke) {
        val args = invoke.getArgs()
        val oldName = args.getString("oldName")
        val newName = args.getString("newName")
        
        if (oldName == null || newName == null) {
            invoke.reject("Missing oldName or newName")
            return
        }
        
        try {
            val oldFile = File(activity.filesDir, oldName)
            val newFile = File(activity.filesDir, newName)
            
            if (!oldFile.exists()) {
                invoke.reject("Source file not found")
                return
            }
            
            if (oldFile.renameTo(newFile)) {
                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } else {
                invoke.reject("Failed to rename file")
            }
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to rename file", e)
            invoke.reject("Failed to rename file: ${e.message}")
        }
    }
    
    @Command
    fun deleteFile(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName")
        
        if (fileName == null) {
            invoke.reject("Missing fileName")
            return
        }
        
        try {
            val file = File(activity.filesDir, fileName)
            
            if (file.exists() && file.delete()) {
                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } else {
                invoke.reject("Failed to delete file")
            }
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to delete file", e)
            invoke.reject("Failed to delete file: ${e.message}")
        }
    }
    
    @Command
    fun fileExists(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName")
        
        if (fileName == null) {
            invoke.reject("Missing fileName")
            return
        }
        
        try {
            val file = File(activity.filesDir, fileName)
            val exists = file.exists()
            
            android.util.Log.d("YellowPlugin", "Checking if file exists: $fileName, result: $exists")
            
            val result = JSObject()
            result.put("exists", exists)
            invoke.resolve(result)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to check file existence", e)
            invoke.reject("Failed to check file existence: ${e.message}")
        }
    }
    
    @Command
    fun openSaveDialog(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName") ?: "download"
        val mimeType = args.getString("mimeType") ?: "application/octet-stream"

        android.util.Log.d("YellowPlugin", "Opening save dialog for file: $fileName, mimeType: $mimeType")

        try {
            pendingInvoke = invoke
            pendingFileName = fileName
            pendingMimeType = mimeType


            android.util.Log.d("YellowPlugin", "Creating intent for ACTION_CREATE_DOCUMENT")
            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = mimeType
                putExtra(Intent.EXTRA_TITLE, fileName)
                
                // Add appropriate directories based on mime type
                when {
                    mimeType.startsWith("image/") -> putExtra("android.provider.extra.INITIAL_URI", MediaStore.Images.Media.EXTERNAL_CONTENT_URI)
                    mimeType.startsWith("video/") -> putExtra("android.provider.extra.INITIAL_URI", MediaStore.Video.Media.EXTERNAL_CONTENT_URI)
                    mimeType.startsWith("audio/") -> putExtra("android.provider.extra.INITIAL_URI", MediaStore.Audio.Media.EXTERNAL_CONTENT_URI)
                    else -> putExtra("android.provider.extra.INITIAL_URI", MediaStore.Downloads.EXTERNAL_CONTENT_URI)
                }
            }

            android.util.Log.d("YellowPlugin", "Opening save dialog for file: $fileName, mimeType: $mimeType")
            startActivityForResult(invoke, intent, "handleSaveDialogResult")
            android.util.Log.d("YellowPlugin", "Save dialog started, waiting for result")
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to open save dialog", e)
            invoke.reject("Failed to open save dialog: ${e.message}")
        }
    }
    
    @Command
    fun saveFileToUri(invoke: Invoke) {
        val args = invoke.getArgs()
        val filePath = args.getString("filePath")
        val uriString = args.getString("uri")
        
        if (filePath == null || uriString == null) {
            invoke.reject("Missing filePath or uri")
            return
        }

        android.util.Log.d("YellowPlugin", "Saving file from $filePath to URI: $uriString")

        try {
            val sourceFile = File(activity.filesDir, filePath)
            if (!sourceFile.exists()) {
                invoke.reject("Source file not found")
                return
            }
            
            val uri = Uri.parse(uriString)

            android.util.Log.d("YellowPlugin", "Opening output stream for URI: $uri")
            
            var bytesCopied: Long = 0
            activity.contentResolver.openOutputStream(uri)?.use { outputStream ->
                sourceFile.inputStream().use { inputStream ->
                    bytesCopied = inputStream.copyTo(outputStream)
                }
            }
            android.util.Log.d("YellowPlugin", "Copied $bytesCopied bytes from ${sourceFile.name} to URI: $uri")
            
            val ret = JSObject()
            ret.put("success", true)
            invoke.resolve(ret)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to save file to URI", e)
            invoke.reject("Failed to save file: ${e.message}")
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
}
