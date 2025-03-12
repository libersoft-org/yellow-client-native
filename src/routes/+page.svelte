<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let notificationTitle = $state("Test Notification");
  let notificationMessage = $state("This is a test notification message");
  let notificationCount = $state(0);
  let notificationDuration = $state(5);
  let notificationClicks = $state<string[]>([]);
  let notificationHistory = $state<any[]>([]);
  let showHistory = $state(false);
  let windowPoolStatus = $state<any>(null);
  let showPoolStatus = $state(false);



  async function testDebug()
  {
    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;

    let notificationId = '';

    // Debug flag
    const DEBUG = true;

    // Debug logging function
    function debugLog(...args) {
      if (DEBUG) {
        console.log('[NOTIFICATION DEBUG]', ...args);
        // Send log to main process
        window.__TAURI__.event.emit('my-log', {
          level: 'debug',
          message: args.map(arg =>
                  typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
          ).join(' ')
        });
      }
    }

    // Log when the script starts
    debugLog('Main window initialized, waiting for events...');


    listen('notification-data', (event) => {
      debugLog('+page received notification-data event:', event);
    });

  }

  onMount(async () => {
    await testDebug();

    // Listen for notification clicks
    await listen("notification-clicked", (event) => {
      const data = event.payload as { id: string };
      notificationClicks = [...notificationClicks, `Clicked notification: ${data.id}`];
    });
    
    // Load notification history and window pool status
    await loadNotificationHistory();
    await loadWindowPoolStatus();
    
    // Refresh window pool status periodically
    const poolStatusInterval = setInterval(loadWindowPoolStatus, 5000);
    
    return () => {
      clearInterval(poolStatusInterval);
    };
  });
  
  async function loadNotificationHistory() {
    try {
      notificationHistory = await invoke("get_notification_history");
      console.log("Loaded notification history:", notificationHistory);
    } catch (error) {
      console.error("Failed to load notification history:", error);
    }
  }
  
  async function loadWindowPoolStatus() {
    try {
      windowPoolStatus = await invoke("get_window_pool_status");
      console.log("Window pool status:", windowPoolStatus);
    } catch (error) {
      console.error("Failed to load window pool status:", error);
    }
  }

  async function setscale(event: Event) {
    await getCurrentWebview().setZoom(1.2);
    console.log(await getCurrentWebview());
    await getCurrentWebview().setZoomFactor(2.2);
    event.preventDefault();
  }

  async function showNotification() {
    try {
      const notificationId = await invoke("create_notification", {
        title: notificationTitle,
        message: notificationMessage + ` (${notificationCount++})`,
        duration: notificationDuration,
        notificationType: 'new_message'
      });
      console.log("Created notification:", notificationId);
    } catch (error) {
      console.error("Failed to create notification:", error);
    }
  }
</script>

<main class="container">

  <form class="row" onsubmit={setscale}>
    <button type="submit">setscale</button>
  </form>

  <div class="notification-section">
    <h2>Notifications</h2>
    <div class="notification-form">
      <div class="form-group">
        <label for="notification-title">Title:</label>
        <input 
          id="notification-title" 
          placeholder="Notification title" 
          bind:value={notificationTitle} 
        />
      </div>
      
      <div class="form-group">
        <label for="notification-message">Message:</label>
        <textarea 
          id="notification-message" 
          placeholder="Notification message" 
          bind:value={notificationMessage}
        ></textarea>
      </div>
      
      <div class="form-group">
        <label for="notification-duration">Duration (seconds):</label>
        <input 
          id="notification-duration" 
          type="number" 
          min="1" 
          max="30" 
          bind:value={notificationDuration} 
        />
      </div>
      
      <button type="button" onclick={showNotification}>
        Show Notification
      </button>
      
      <button type="button" onclick={() => showHistory = !showHistory}>
        {showHistory ? 'Hide History' : 'Show History'}
      </button>
      
      <button type="button" onclick={() => showPoolStatus = !showPoolStatus}>
        {showPoolStatus ? 'Hide Window Pool' : 'Show Window Pool'}
      </button>
    </div>
    
    {#if showHistory}
      <div class="notification-history">
        <h3>Notification History</h3>
        {#if notificationHistory.length === 0}
          <p>No notification history available.</p>
        {:else}
          <ul>
            {#each notificationHistory as notification}
              <li>
                <strong>{notification.title}</strong>
                <p>{notification.message}</p>
                <small>
                  {notification.timestamp 
                    ? new Date(notification.timestamp * 1000).toLocaleString() 
                    : 'No timestamp'}
                </small>
              </li>
            {/each}
          </ul>
        {/if}
        <button type="button" onclick={loadNotificationHistory}>Refresh History</button>
      </div>
    {/if}
    
    {#if showPoolStatus}
      <div class="window-pool-status">
        <h3>Window Pool Status</h3>
        {#if !windowPoolStatus}
          <p>Loading window pool status...</p>
        {:else}
          <div class="pool-stats">
            <div class="stat-item">
              <span class="stat-label">Pooled Windows:</span>
              <span class="stat-value">{windowPoolStatus.pooled_windows}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Total Windows:</span>
              <span class="stat-value">{windowPoolStatus.total_windows}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Max Windows:</span>
              <span class="stat-value">{windowPoolStatus.max_windows}</span>
            </div>
          </div>
          
          {#if windowPoolStatus.window_ids && windowPoolStatus.window_ids.length > 0}
            <div class="pool-window-list">
              <h4>Pooled Window IDs:</h4>
              <ul>
                {#each windowPoolStatus.window_ids as windowId}
                  <li>{windowId}</li>
                {/each}
              </ul>
            </div>
          {:else}
            <p>No windows in pool.</p>
          {/if}
        {/if}
        <button type="button" onclick={loadWindowPoolStatus}>Refresh Pool Status</button>
      </div>
    {/if}

    {#if notificationClicks.length > 0}
      <div class="notification-clicks">
        <h3>Notification Clicks:</h3>
        <ul>
          {#each notificationClicks as click}
            <li>{click}</li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
</main>

<style>
.notification-section {
  margin-top: 2rem;
  padding: 1rem;
  border-top: 1px solid #ddd;
  text-align: left;
}

.notification-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 500px;
  margin: 0 auto;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

textarea {
  min-height: 80px;
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-family: inherit;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  resize: vertical;
}

.notification-clicks, .notification-history, .window-pool-status {
  margin-top: 1.5rem;
  padding: 1rem;
  background-color: #f5f5f5;
  border-radius: 8px;
  overflow: auto;
  max-height: 300px;
}

.notification-clicks ul, .notification-history ul, .window-pool-status ul {
  margin: 0;
  padding-left: 1.5rem;
  list-style-type: none;
}

.notification-clicks li, .notification-history li, .window-pool-status li {
  margin-bottom: 0.5rem;
  padding: 0.5rem;
  border-bottom: 1px solid #ddd;
}

.notification-history li {
  background-color: white;
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 10px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.notification-history small {
  color: #666;
  display: block;
  margin-top: 5px;
}

.pool-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  margin-bottom: 1rem;
}

.stat-item {
  background-color: white;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-label {
  font-size: 0.8rem;
  color: #666;
}

.stat-value {
  font-size: 1.2rem;
  font-weight: bold;
  color: #333;
}

.pool-window-list {
  margin-top: 1rem;
}

.pool-window-list h4 {
  margin-bottom: 0.5rem;
}

@media (prefers-color-scheme: dark) {
  .notification-section {
    border-top: 1px solid #444;
  }
  
  .notification-clicks {
    background-color: #333;
  }
  
  textarea {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
