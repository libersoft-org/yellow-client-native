package org.libersoft.yellowplugin

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat

class YellowForegroundService : Service() {
    companion object {
        const val CHANNEL_ID = "yellow_foreground_service"
        const val NOTIFICATION_ID = 1001
        const val ACTION_START = "org.libersoft.yellowplugin.START_SERVICE"
        const val ACTION_STOP = "org.libersoft.yellowplugin.STOP_SERVICE"
        const val EXTRA_TITLE = "title"
        const val EXTRA_MESSAGE = "message"
        private const val TAG = "YellowForegroundService"
        
        @Volatile
        private var isServiceRunning = false
        
        fun isRunning(): Boolean = isServiceRunning
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Service onCreate")
        isServiceRunning = true
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "Service onStartCommand - action: ${intent?.action}")
        
        when (intent?.action) {
            ACTION_START -> {
                val title = intent.getStringExtra(EXTRA_TITLE) ?: "Yellow Service"
                val message = intent.getStringExtra(EXTRA_MESSAGE) ?: "Service is running"
                startForegroundService(title, message)
            }
            ACTION_STOP -> {
                stopForegroundService()
            }
            else -> {
                // Handle service restart after being killed by system
                startForegroundService("Yellow Service", "Service is running")
            }
        }
        
        // START_STICKY ensures service restarts if killed by system
        return START_STICKY
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Yellow Foreground Service",
                NotificationManager.IMPORTANCE_DEFAULT  // Changed from LOW to DEFAULT
            ).apply {
                description = "Notification channel for Yellow foreground service"
                setShowBadge(false)
                setSound(null, null)  // Disable sound for this channel
            }
            
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager?.createNotificationChannel(channel)
            Log.d(TAG, "Notification channel created")
        }
    }

    private fun startForegroundService(title: String, message: String) {
        Log.d(TAG, "Starting foreground service - title: $title, message: $message")
        
        // Create intent to launch app when notification is clicked
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            launchIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        // Get app's launcher icon
        val appInfo = applicationInfo
        val smallIcon = appInfo.icon
        
        val notificationBuilder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(title)
            .setContentText(message)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
        
        // Set small icon - use app icon if available, otherwise fallback
        if (smallIcon != 0) {
            notificationBuilder.setSmallIcon(smallIcon)
        } else {
            // Fallback to a simple drawable
            notificationBuilder.setSmallIcon(android.R.drawable.stat_notify_sync)
        }
        
        val notification = notificationBuilder.build()
        
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            Log.d(TAG, "Foreground service started successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start foreground service", e)
            throw e
        }
    }

    private fun stopForegroundService() {
        Log.d(TAG, "Stopping foreground service")
        stopForeground(true)
        stopSelf()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "Service onDestroy")
        isServiceRunning = false
    }
}