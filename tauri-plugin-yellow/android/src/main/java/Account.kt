package org.libersoft.yellowplugin

import android.content.Context
import android.util.Log
import okhttp3.*
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.*
import com.github.taoweiji.quickjs.JSContext
import com.github.taoweiji.quickjs.JSFunction
import com.github.taoweiji.quickjs.JSValue
import java.net.URL
import java.net.HttpURLConnection

class Account(
    private val context: Context,
    val id: String,
    private val credentials: AccountCredentials,
    private val settings: AccountSettings,
    private val messageHandler: (String, JSONObject) -> Unit
) {
    companion object {
        private const val TAG = "Account"
        private const val PING_INTERVAL_MS = 10000L
        private const val RECONNECT_DELAY_MS = 5000L
        private const val SERVICE_FILE_PATH = "tmp/messages-service.js" // Path in static assets
    }
    
    data class AccountCredentials(
        val server: String,
        val address: String,
        val password: String,
        val retryNonce: Int = 0
    )
    
    data class AccountSettings(
        val displayName: String? = null,
        val modulesEnabled: List<String> = emptyList()
    )
    
    private var webSocket: WebSocket? = null
    private val okHttpClient = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .build()
    
    private var socketId = 0
    private var sessionId: String? = null
    private var wsGuid: String? = null
    private var lastCommsTs = 0L
    private var lastTransmissionTs = 0L
    
    var isEnabled = false
        private set
    private var status = "Disabled"
    private var error: String? = null
    
    private val coroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var pingJob: Job? = null
    private var reconnectJob: Job? = null
    
    // JavaScript isolate for module execution
    private var jsContext: JSContext? = null
    private var moduleServiceCode: String? = null
    
    fun enable() {
        Log.d(TAG, "Enabling account: $id")
        isEnabled = true
        status = "Enabled"
        connect()
    }
    
    fun disable() {
        Log.d(TAG, "Disabling account: $id")
        isEnabled = false
        status = "Disabled"
        disconnect()
        cleanupJSContext()
    }
    
    private fun connect() {
        if (!isEnabled) return
        
        disconnect() // Clean up any existing connection
        socketId++
        val currentSocketId = socketId
        
        status = "Connecting..."
        Log.d(TAG, "Connecting to WebSocket: ${credentials.server}")
        
        val request = Request.Builder()
            .url(credentials.server)
            .build()
        
        webSocket = okHttpClient.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                if (socketId != currentSocketId) {
                    Log.d(TAG, "Socket ID changed, ignoring open event")
                    return
                }
                
                Log.d(TAG, "WebSocket opened for account: $id")
                status = "Connected, logging in..."
                lastCommsTs = System.currentTimeMillis()
                
                sendLogin()
                startPingTimer()
            }
            
            override fun onMessage(webSocket: WebSocket, text: String) {
                if (socketId != currentSocketId) {
                    Log.d(TAG, "Socket ID changed, ignoring message")
                    return
                }
                
                lastCommsTs = System.currentTimeMillis()
                handleMessage(text)
            }
            
            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                if (socketId != currentSocketId) {
                    Log.d(TAG, "Socket ID changed, ignoring failure")
                    return
                }
                
                Log.e(TAG, "WebSocket failure for account $id", t)
                error = t.message
                status = "Connection failed"
                scheduleReconnect()
            }
            
            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                if (socketId != currentSocketId) {
                    Log.d(TAG, "Socket ID changed, ignoring close event")
                    return
                }
                
                Log.d(TAG, "WebSocket closed for account $id: $code - $reason")
                sessionId = null
                scheduleReconnect()
            }
        })
    }
    
    private fun disconnect() {
        webSocket?.close(1000, "Disconnecting")
        webSocket = null
        pingJob?.cancel()
        reconnectJob?.cancel()
    }
    
    private fun sendLogin() {
        val loginData = JSONObject().apply {
            put("target", "core")
            put("command", "login")
            put("params", JSONObject().apply {
                put("address", credentials.address)
                put("password", credentials.password)
            })
            put("requestID", generateRequestId())
        }
        
        webSocket?.send(loginData.toString())
        lastTransmissionTs = System.currentTimeMillis()
    }
    
    private fun handleMessage(text: String) {
        try {
            val message = JSONObject(text)
            
            // Forward all messages to frontend
            messageHandler(id, message)
            
            when {
                message.has("command") && message.getString("command") == "login" -> {
                    handleLoginResponse(message)
                }
                message.has("module") && message.getString("module") == "org.libersoft.messages" -> {
                    // Also forward to JavaScript isolate
                    forwardToJSContext(message)
                }
                else -> {
                    Log.d(TAG, "Received message: $text")
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error handling message", e)
        }
    }
    
    private fun handleLoginResponse(message: JSONObject) {
        if (message.getBoolean("error")) {
            error = message.optString("message", "Login failed")
            status = "Login failed"
            Log.e(TAG, "Login failed: $error")
            scheduleReconnect()
        } else {
            val data = message.getJSONObject("data")
            sessionId = data.getString("sessionID")
            wsGuid = data.getString("wsGuid")
            status = "Connected"
            error = null
            
            Log.d(TAG, "Login successful for account $id")
            
            // Initialize JavaScript context after successful login
            initializeJSContext()
            
            // Subscribe to events
            subscribeToEvents()
        }
    }
    
    private fun subscribeToEvents() {
        // Subscribe to message events
        val subscribeData = JSONObject().apply {
            put("target", "org.libersoft.messages")
            put("command", "subscribe")
            put("params", JSONObject().apply {
                put("events", listOf("message", "conversation", "reaction"))
            })
            put("requestID", generateRequestId())
            put("sessionID", sessionId)
        }
        
        webSocket?.send(subscribeData.toString())
    }
    
    private fun startPingTimer() {
        pingJob?.cancel()
        pingJob = coroutineScope.launch {
            while (isActive) {
                delay(PING_INTERVAL_MS)
                
                if (webSocket == null) {
                    status = "Retrying..."
                    error = "Not connected"
                    connect()
                    break
                }
                
                sendPing()
                
                // Check for timeout
                val noCommsMs = System.currentTimeMillis() - lastCommsTs
                if (lastTransmissionTs > 0 && noCommsMs > 60000 + (System.currentTimeMillis() - lastTransmissionTs)) {
                    Log.d(TAG, "No comms for ${noCommsMs / 1000} seconds, reconnecting...")
                    status = "Retrying..."
                    error = "Connection timeout"
                    connect()
                    break
                }
            }
        }
    }
    
    private fun sendPing() {
        val pingData = JSONObject().apply {
            put("target", "core")
            put("command", "ping")
            put("params", JSONObject())
            put("requestID", generateRequestId())
            put("sessionID", sessionId)
        }
        
        webSocket?.send(pingData.toString())
        lastTransmissionTs = System.currentTimeMillis()
    }
    
    private fun scheduleReconnect() {
        if (!isEnabled) return
        
        reconnectJob?.cancel()
        reconnectJob = coroutineScope.launch {
            delay(RECONNECT_DELAY_MS)
            if (isEnabled) {
                connect()
            }
        }
    }
    
    private fun initializeJSContext() {
        coroutineScope.launch {
            try {
                // Load module service code if not already loaded
                if (moduleServiceCode == null) {
                    moduleServiceCode = loadModuleService()
                }
                
                // Create QuickJS context
                jsContext = JSContext.create()
                
                jsContext?.let { ctx ->
                    // Set up Kotlin callback functions
                    ctx.set("__kotlinSendMessage", JSFunction { _, args ->
                        if (args.isNotEmpty()) {
                            val dataStr = args[0].toString()
                            sendMessageFromJS(dataStr)
                        }
                        JSValue.Undefined
                    })
                    
                    ctx.set("__kotlinLog", JSFunction { _, args ->
                        if (args.size >= 2) {
                            val level = args[0].toString()
                            val message = args[1].toString()
                            Log.d("$TAG-JS", "[$level] $message")
                        }
                        JSValue.Undefined
                    })
                    
                    // Inject global constants and functions expected by the service
                    ctx.evaluate("""
                        // Set TAURI_MOBILE flag
                        globalThis.TAURI_MOBILE = true;
                        
                        // Inject send() function that routes through WebSocket
                        globalThis.send = function(acc, account, target, command, params, sendSessionID, callback, quiet) {
                            const message = {
                                target: target,
                                command: command,
                                params: params || {},
                                sendSessionID: sendSessionID !== false,
                                quiet: quiet === true
                            };
                            
                            // Send through WebSocket and handle callback
                            __kotlinSendMessage(JSON.stringify(message));
                            
                            // For now, call callback immediately with success
                            // TODO: Implement proper request/response tracking
                            if (callback) {
                                setTimeout(() => {
                                    callback(message, { error: false, data: {} });
                                }, 0);
                            }
                        };
                        
                        // Kotlin bridge for service communication
                        globalThis.KotlinBridge = {
                            sendMessage: function(data) {
                                return __kotlinSendMessage(data);
                            },
                            log: function(level, message) {
                                __kotlinLog(level, message);
                            }
                        };
                    """.trimIndent())
                    
                    // Load the module service
                    moduleServiceCode?.let { code ->
                        ctx.evaluate(code)
                        
                        // Initialize the service
                        ctx.evaluate("""
                            if (typeof YellowMessagesService !== 'undefined' && YellowMessagesService.init) {
                                YellowMessagesService.init({
                                    accountId: '${id}',
                                    server: '${credentials.server}',
                                    address: '${credentials.address}'
                                });
                            }
                        """.trimIndent())
                    }
                }
                
                Log.d(TAG, "JavaScript context initialized for account $id")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to initialize JS context", e)
                error = "Failed to initialize module service"
            }
        }
    }
    
    private suspend fun loadModuleService(): String {
        return withContext(Dispatchers.IO) {
            try {
                // First try to load from assets (for bundled service file)
                try {
                    context.assets.open(SERVICE_FILE_PATH).use { inputStream ->
                        inputStream.bufferedReader().use { it.readText() }
                    }
                } catch (e: Exception) {
                    Log.w(TAG, "Could not load service from assets, trying development approach", e)
                    
                    // Fallback: try to load from development server (when using bun run dev)
                    // This allows for development without rebuilding the entire app
                    val devUrl = "http://localhost:5173/$SERVICE_FILE_PATH"
                    val url = URL(devUrl)
                    val connection = url.openConnection() as HttpURLConnection
                    connection.requestMethod = "GET"
                    connection.connectTimeout = 5000
                    connection.readTimeout = 5000
                    
                    val responseCode = connection.responseCode
                    if (responseCode == HttpURLConnection.HTTP_OK) {
                        connection.inputStream.bufferedReader().use { it.readText() }
                    } else {
                        throw Exception("HTTP error code: $responseCode when loading from dev server")
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load module service from both assets and dev server", e)
                // Return empty service as fallback
                """
                console.warn('Module service failed to load, using fallback');
                globalThis.YellowMessagesService = {
                    init: function(config) { console.log('Fallback service init:', config); },
                    handleMessage: function(message) { console.log('Fallback service message:', message); },
                    destroy: function() { console.log('Fallback service destroy'); }
                };
                """.trimIndent()
            }
        }
    }
    
    private fun forwardToJSContext(message: JSONObject) {
        jsContext?.let { ctx ->
            try {
                ctx.evaluate("""
                    if (typeof YellowMessagesService !== 'undefined' && YellowMessagesService.handleMessage) {
                        YellowMessagesService.handleMessage(${message.toString()});
                    }
                """.trimIndent())
            } catch (e: Exception) {
                Log.e(TAG, "Error forwarding message to JS context", e)
            }
        }
    }
    
    private fun sendMessageFromJS(dataStr: String) {
        try {
            val data = JSONObject(dataStr)
            // Add session ID if not present
            if (!data.has("sessionID")) {
                data.put("sessionID", sessionId)
            }
            
            webSocket?.send(data.toString())
            lastTransmissionTs = System.currentTimeMillis()
        } catch (e: Exception) {
            Log.e(TAG, "Error sending message from JS", e)
        }
    }
    
    private fun cleanupJSContext() {
        jsContext?.close()
        jsContext = null
    }
    
    private fun generateRequestId(): String {
        return "req_${System.currentTimeMillis()}_${(Math.random() * 1000000).toInt()}"
    }
    
    fun sendMessage(message: JSONObject) {
        if (webSocket == null) {
            Log.e(TAG, "Cannot send message - WebSocket is null")
            return
        }
        
        webSocket?.send(message.toString())
        lastTransmissionTs = System.currentTimeMillis()
    }
    
    fun destroy() {
        disable()
        coroutineScope.cancel()
    }
}