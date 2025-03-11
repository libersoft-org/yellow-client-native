<script>
    import { onMount } from "svelte";

    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;

    let notificationId = '';
    let windowLabel = '';
    const DEBUG = true;
    let title = 'Notification Title';
    let message = 'Notification message goes here.';
    let duration = 0;
    let progressActive = false;
    let sizeInfo = 'Size: calculating...';

    function localLog(message) {
        const logElement = document.getElementById('log');
        if (logElement) {
            const logLine = document.createElement('div');
            logLine.textContent = message;
            logElement.appendChild(logLine);
        }
    }

    // Debug logging function
    function debugLog(...args) {
        if (DEBUG) {
            console.log('[NOTIFICATION DEBUG]', ...args);
            try {
                localLog(JSON.stringify(args));

                // Send log to main process
                window.__TAURI__.event.emit('my-log', {
                    level: 'debug',
                    message: 'NOTIFICATION WINDOW: ' + args.map(arg =>
                        typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                    ).join(' ')
                });
            } catch (e) {
                console.error('Failed to emit log event:', e);
            }
        }
    }
    
    // Function to update size debug info
    function updateSizeDebug() {
        const width = window.innerWidth;
        const height = window.innerHeight;
        const outerWidth = window.outerWidth || width;
        const outerHeight = window.outerHeight || height;
        const devicePixelRatio = window.devicePixelRatio || 1;
        
        // Get size from Rust as well
        invoke('get_window_size')
            .then(([rustWidth, rustHeight]) => {
                sizeInfo = `JS: ${width}×${height} | Rust: ${rustWidth}×${rustHeight} | DPI: ${devicePixelRatio.toFixed(2)}`;
                debugLog('Window size:', { 
                    js: { width, height, outerWidth, outerHeight, devicePixelRatio },
                    rust: { width: rustWidth, height: rustHeight }
                });
            })
            .catch(err => {
                sizeInfo = `JS: ${width}×${height} | DPI: ${devicePixelRatio.toFixed(2)} | Rust: error`;
                debugLog('Error getting window size from Rust:', err);
            });
        
        return { width, height, outerWidth, outerHeight, devicePixelRatio };
    }

    function closeWithAnimation() {
        document.body.style.animation = 'fade-out 0.2s ease-out forwards';

        setTimeout(() => {
            invoke('close_notification', { notification_id: notificationId });
        }, 200);
    }

    function handleContainerClick() {
        // Emit an event that the notification was clicked with structured payload
        window.__TAURI__.event.emit('notification-clicked', {
            id: notificationId,
            window_label: windowLabel,
            action: 'clicked',
            timestamp: new Date().toISOString()
        });

        // Close the notification with animation
        closeWithAnimation();
    }

    function handleCloseButtonClick(e) {
        e.stopPropagation(); // Prevent triggering the container click
        closeWithAnimation();
    }

    onMount(async () => {
        // Immediately log that the notification window has loaded
        debugLog('[NOTIFICATION DEBUG] ❤Window loaded at', new Date().toISOString());
        
        // Get scale factor from Rust
        invoke('get_scale_factor')
            .then(scaleFactor => {
                debugLog('Scale factor from Rust:', scaleFactor);
                document.documentElement.style.setProperty('--scale-factor', scaleFactor);
            })
            .catch(err => {
                debugLog('Error getting scale factor from Rust:', err);
            });
        
        // Update size debug info immediately and periodically
        updateSizeDebug();
        const sizeInterval = setInterval(updateSizeDebug, 10000);
        
        // Set initial title to show window is loading
        title = 'Loading...';

        // Log all available Tauri APIs
        debugLog('Available Tauri APIs:', Object.keys(window.__TAURI__ || {}));

        // Set a visual indicator that we're initializing
        title = 'Initializing...';
        document.body.style.border = '2px solid orange';

        // Wait a moment before calling the notification_ready command to ensure Tauri is ready
        setTimeout(() => {
            debugLog('Window is ready, calling notification_ready command');

            try {
                // Use invoke to call the notification_ready command
                invoke('notification_ready')
                    .then(notificationData => {
                        debugLog('✅ RECEIVED notification data from command:', JSON.stringify(notificationData));

                        // Set a visual indicator immediately
                        title = 'DATA RECEIVED';
                        document.body.style.border = '2px solid blue';

                        try {
                            notificationId = notificationData.id;
                            windowLabel = notificationData.window_label;

                            debugLog('Notification ', JSON.stringify(notificationData));
                            debugLog('Notification ID:', notificationId);
                            debugLog('Window Label:', windowLabel);

                            title = notificationData.title;
                            message = notificationData.message;
                            duration = notificationData.duration;

                            // Set the progress bar animation duration and activate it
                            const progressBar = document.getElementById('progress');
                            if (progressBar) {
                                progressBar.style.animationDuration = `${duration}s`;
                                
                                // Ensure the animation starts from the beginning by removing and re-adding the element
                                const parent = progressBar.parentNode;
                                const newProgressBar = progressBar.cloneNode(true);
                                parent.removeChild(progressBar);
                                parent.appendChild(newProgressBar);
                                
                                // Trigger the animation after a small delay to ensure it's properly reset
                                setTimeout(() => {
                                    const updatedProgressBar = document.getElementById('progress');
                                    if (updatedProgressBar) {
                                        progressActive = true;
                                        
                                        // Set timeout to close the notification when duration expires
                                        setTimeout(() => {
                                            // Emit an event that the notification is closing due to timeout
                                            window.__TAURI__.event.emit('notification-timeout', {
                                                id: notificationId,
                                                window_label: windowLabel,
                                                action: 'timeout',
                                                timestamp: new Date().toISOString()
                                            });
                                            
                                            // Then close with animation
                                            closeWithAnimation();
                                        }, duration * 1000);
                                    }
                                }, 50);
                            }

                            // Add a visual indicator that the data was received
                            document.body.style.border = '2px solid green';
                            setTimeout(() => {
                                document.body.style.border = '1px solid rgba(0, 0, 0, 0.1)';
                            }, 500);

                            // Send acknowledgment that we received and processed the notification data
                            window.__TAURI__.event.emit('notification-data-received', {
                                id: notificationId,
                                window_label: windowLabel,
                                status: 'success'
                            });

                            debugLog('Notification data applied successfully:', JSON.stringify(notificationData));
                        } catch (error) {
                            debugLog('Error processing notification:', error.toString());
                            // Visual indicator for error
                            document.body.style.border = '2px solid red';

                            // Send error acknowledgment
                            window.__TAURI__.event.emit('notification-data-received', {
                                id: notificationId || 'unknown',
                                window_label: windowLabel || 'unknown',
                                status: 'error',
                                error: error.toString()
                            });
                        }
                    })
                    .catch(error => {
                        debugLog('Error calling notification_ready command:', error.toString());
                        title = 'Error: ' + error.message;
                        document.body.style.border = '2px solid red';
                    });

            } catch (error) {
                debugLog('Error calling notification_ready command:', error.toString());
                title = 'Error: ' + error.message;
                document.body.style.border = '2px solid red';
            }
        }, 100);

        return () => {
            clearInterval(sizeInterval);
        };
    });
</script>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        background-color: rgba(255, 255, 255, 0.95);
        border-radius: 8px;
        overflow: hidden;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        border: 1px solid rgba(0, 0, 0, 0.1);
        height: 100vh;
        width: 100vw;
        display: flex;
        flex-direction: column;
        animation: fade-in 0.1s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(-10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .notification-container {
        padding: 16px;
        flex: 1;
        display: flex;
        flex-direction: column;
        cursor: pointer;
    }

    .notification-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }

    .notification-title {
        font-weight: bold;
        font-size: 18px;
        margin: 0;
        color: #333;
    }

    .close-button {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 18px;
        color: #999;
    }

    .notification-message {
        font-size: 16px;
        color: #555;
        flex: 1;
        margin-top: 10px;
        line-height: 1.4;
    }

    .progress-bar {
        min-height: 4px;
        background-color: #e0e0e0;
        width: 100%;
        position: relative;
        overflow: hidden;
    }

    .progress {
        position: absolute;
        height: 100%;
        background-color: #4a86e8;
        width: 100%;
        animation-name: progress;
        animation-timing-function: linear;
        animation-fill-mode: forwards;
        animation-play-state: paused; /* Start paused */
    }
    
    .progress.active {
        animation-play-state: running; /* Will be activated via JS */
    }

    .log {
        color: #fff;
    }

    .size-debug {
        position: fixed;
        bottom: 5px;
        right: 5px;
        background: rgba(0,0,0,0.7);
        color: white;
        padding: 3px 6px;
        font-size: 10px;
        border-radius: 3px;
        z-index: 9999;
    }

    @keyframes progress {
        from { width: 100%; }
        to { width: 0%; }
    }

    @keyframes fade-out {
        from { opacity: 1; transform: translateY(0); }
        to { opacity: 0; transform: translateY(-10px); }
    }

    /* Dark mode support */
    @media (prefers-color-scheme: dark) {
        :global(body) {
            background-color: rgba(45, 45, 45, 0.9);
        }

        .notification-title {
            color: #f0f0f0;
        }

        .notification-message {
            color: #d0d0d0;
        }

        .progress-bar {
            background-color: #444;
        }
    }
</style>

<div class="progress-bar">
    <div class="progress" id="progress" class:active={progressActive}></div>
</div>
<div class="notification-container" id="notification-container" on:click={handleContainerClick}>
    <div class="notification-header">
        <h3 class="notification-title" id="notification-title">{title}</h3>
        <button class="close-button" id="close-button" on:click={handleCloseButtonClick}>×</button>
    </div>
    <div class="notification-message" id="notification-message">
        {message}
    </div>
    <div class="log" id="log">
        <!-- Debug logs will appear here -->
    </div>
    <div class="size-debug" id="size-debug">
        {sizeInfo}
    </div>
</div>
