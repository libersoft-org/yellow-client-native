<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let name = $state("");
  let greetMsg = $state("");
  let notificationTitle = $state("Test Notification");
  let notificationMessage = $state("This is a test notification message");
  let notificationDuration = $state(5);
  let notificationClicks = $state<string[]>([]);

  onMount(async () => {
    // Listen for notification clicks
    await listen("notification-clicked", (event) => {
      const data = event.payload as { id: string };
      notificationClicks = [...notificationClicks, `Clicked notification: ${data.id}`];
    });
  });

  async function greet(event: Event) {
    await getCurrentWebview().setZoom(1.2);
    console.log(await getCurrentWebview());
    await getCurrentWebview().setZoomFactor(2.2);

    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }

  async function showNotification() {
    try {
      const notificationId = await invoke("create_notification", {
        title: notificationTitle,
        message: notificationMessage,
        duration: notificationDuration
      });
      console.log("Created notification:", notificationId);
    } catch (error) {
      console.error("Failed to create notification:", error);
    }
  }
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <form class="row" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>

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
      
      <button type="button" on:click={showNotification}>
        Show Notification
      </button>
    </div>

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
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

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

.notification-clicks {
  margin-top: 1.5rem;
  padding: 1rem;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.notification-clicks ul {
  margin: 0;
  padding-left: 1.5rem;
}

.notification-clicks li {
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

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
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

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
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
