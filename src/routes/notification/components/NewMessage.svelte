<script>
    export let title = 'Notification Title';
    export let message = 'Notification message goes here.';
    export let onClose = () => {};
</script>

<style>


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


</style>

<div class="notification-header">
    <h3 class="notification-title" id="notification-title">{title}</h3>
    <button 
        class="close-button" 
        id="close-button" 
        on:click={onClose}
        aria-label="Close notification"
        tabindex="0"
    >×</button>
</div>
<div class="notification-message" id="notification-message">
    {message}
</div>
