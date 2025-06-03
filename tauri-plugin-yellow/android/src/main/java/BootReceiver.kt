package org.libersoft.yellowplugin

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log

class BootReceiver : BroadcastReceiver() {
    companion object {
        private const val TAG = "YellowBootReceiver"
    }

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            Log.d(TAG, "Boot completed, starting foreground service")
            
            val serviceIntent = Intent(context, YellowForegroundService::class.java).apply {
                action = YellowForegroundService.ACTION_START
                putExtra(YellowForegroundService.EXTRA_TITLE, "Yellow Service")
                putExtra(YellowForegroundService.EXTRA_MESSAGE, "Service started on boot")
            }
            
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    context.startForegroundService(serviceIntent)
                } else {
                    context.startService(serviceIntent)
                }
                Log.d(TAG, "Foreground service started successfully on boot")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start service on boot", e)
            }
        }
    }
}