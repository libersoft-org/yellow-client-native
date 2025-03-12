<script>
    import { onMount } from "svelte";
    import NewMessage from './components/NewMessage.svelte';

    const { invoke } = window.__TAURI__.core;
    const { listen } = window.__TAURI__.event;

    const DEBUG = true;
    let windowId = '';
    let notificationId = '';

    // Notification data received from Rust
    let data = null;
    let title = 'Loading...';
    let message = 'Please wait...';
    let duration = 0;
    let notificationType = '';

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

    function handleContainerClick() {
        // Emit notification clicked event
        window.__TAURI__.event.emit('notification-clicked', {
            id: notificationId,
            window_id: windowId,
            timestamp: new Date().toISOString()
        });
        
        // Call close notification
        closeNotification();
    }
    
    // Handle keyboard navigation
    function handleKeyDown(event) {
        if (event.key === 'Escape') {
            handleCloseButtonClick(event);
        } else if (event.key === 'Enter' || event.key === ' ') {
            handleContainerClick();
        }
    }

    function handleCloseButtonClick(e) {
        e.stopPropagation();
        // Emit user closed notification event
        window.__TAURI__.event.emit('user_closed_notification', {
            id: notificationId,
            window_id: windowId,
            timestamp: new Date().toISOString()
        });
        
        // Call close notification
        closeNotification();
    }
    
    function closeNotification() {
        // Call the Rust command to close this notification
        invoke('close_notification', { windowId })
            .catch(err => {
                debugLog('Error closing notification:', err);
            });
    }
    
    function setupProgressBar() {
        if (!duration) return;
        
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
                            window_id: windowId,
                            action: 'timeout',
                            timestamp: new Date().toISOString()
                        });
                        
                        // Close the notification
                        closeNotification();
                    }, duration * 1000);
                }
            }, 50);
        }
    }

    onMount(async () => {
        debugLog('[NOTIFICATION DEBUG] Window loaded at', new Date().toISOString());
        windowId = window.__TAURI__.window.label;
        debugLog('Window ID:', windowId);
        
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
        
        // Listen for notification data events
        await listen('notification-data', (event) => {
            debugLog('Received notification-data event:', event);
            
            try {
                data = event.payload;
                notificationId = data.id;
                title = data.title;
                message = data.message;
                duration = data.duration;
                notificationType = data.notification_type;
                
                debugLog('Notification data applied:', {
                    id: notificationId,
                    title,
                    message,
                    duration,
                    type: notificationType
                });
                
                // Setup progress bar for auto-close
                setupProgressBar();
                
                // Visual indicator that data was received
                document.body.style.border = '2px solid green';
                setTimeout(() => {
                    document.body.style.border = '1px solid rgba(0, 0, 0, 0.1)';
                }, 500);
                
                // Send acknowledgment
                window.__TAURI__.event.emit('notification-data-received', {
                    id: notificationId,
                    window_id: windowId,
                    status: 'success'
                });
            } catch (error) {
                debugLog('Error processing notification data:', error.toString());
                document.body.style.border = '2px solid red';
                
                // Send error acknowledgment
                window.__TAURI__.event.emit('notification-data-received', {
                    window_id: windowId,
                    status: 'error',
                    error: error.toString()
                });
            }
        });
        
        // Notify Rust that the window is ready
        debugLog('Window is ready, calling notification_window_ready command');
        try {
            await invoke('notification_window_ready');
            debugLog('notification_window_ready command completed');
        } catch (error) {
            debugLog('Error calling notification_window_ready command:', error.toString());
            document.body.style.border = '2px solid red';
        }

        return () => {
            clearInterval(sizeInterval);
            window.removeEventListener('keydown', handleKeyDown);
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
    
    /* Focus styles for accessibility */
    :global(:focus) {
        outline: 2px solid #4a86e8;
        outline-offset: 2px;
    }
    
    :global(:focus:not(:focus-visible)) {
        outline: none;
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
        outline: none; /* We'll use custom focus styles */
    }
    
    /* Accessibility - indicate interactive element */
    .notification-container:hover {
        background-color: rgba(0, 0, 0, 0.02);
    }
    
    .notification-container:focus {
        background-color: rgba(0, 0, 0, 0.03);
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

<div 
    class="notification-container" 
    id="notification-container" 
    on:click={handleContainerClick}
    role="alert"
    aria-live="assertive"
    tabindex="0"
    on:keydown={e => e.key === 'Enter' && handleContainerClick()}
>
    {#if duration > 0}
    <div class="progress-bar" aria-hidden="true">
        <div class="progress" id="progress" class:active={progressActive}></div>
    </div>
    {/if}

    {#if data && notificationType === 'new_message'}
        <NewMessage {title} {message} onClose={handleCloseButtonClick} />
    {:else if data}
        <div class="notification-header">
            <h3 class="notification-title" id="notification-title">{title}</h3>
            <button 
                class="close-button" 
                on:click={handleCloseButtonClick}
                aria-label="Close notification"
                tabindex="0"
            >×</button>
        </div>
        <div class="notification-message" id="notification-message">
            {message}
        </div>
    {:else}
        <div class="notification-header">
            <h3 class="notification-title">Loading...</h3>
        </div>
        <div class="notification-message">
            Waiting for notification data...
        </div>
    {/if}

    <div class="log" id="log" aria-hidden="true">
        <!-- Debug logs will appear here -->
    </div>
    <div class="size-debug" id="size-debug" aria-hidden="true">
        {sizeInfo}
    </div>
</div>
