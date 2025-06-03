package org.libersoft.yellowplugin

import android.app.Activity
import android.content.ContentValues
import android.content.Intent
import android.media.MediaScannerConnection
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import java.io.File
import java.io.FileOutputStream

class Files(private val activity: Activity) {
    
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
                android.util.Log.d("YellowPlugin", "API level < 29, using direct file access")

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
    
    fun getFileSize(invoke: Invoke) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName")
        
        if (fileName == null) {
            invoke.reject("Missing fileName")
            return
        }
        
        try {
            val file = File(activity.filesDir, fileName)
            
            if (!file.exists()) {
                invoke.reject("File not found")
                return
            }
            
            val size = file.length()
            android.util.Log.d("YellowPlugin", "File size for $fileName: $size bytes")
            
            val result = JSObject()
            result.put("size", size)
            invoke.resolve(result)
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to get file size", e)
            invoke.reject("Failed to get file size: ${e.message}")
        }
    }
    
    fun openSaveDialog(invoke: Invoke, pendingInvokeCallback: (Invoke) -> Unit, startActivityForResultCallback: (Invoke, Intent, String) -> Unit) {
        val args = invoke.getArgs()
        val fileName = args.getString("fileName") ?: "download"
        val mimeType = args.getString("mimeType") ?: "application/octet-stream"

        android.util.Log.d("YellowPlugin", "Opening save dialog for file: $fileName, mimeType: $mimeType")

        try {
            pendingInvokeCallback(invoke)

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
            startActivityForResultCallback(invoke, intent, "handleSaveDialogResult")
            android.util.Log.d("YellowPlugin", "Save dialog started, waiting for result")
        } catch (e: Exception) {
            android.util.Log.e("YellowPlugin", "Failed to open save dialog", e)
            invoke.reject("Failed to open save dialog: ${e.message}")
        }
    }
    
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
}