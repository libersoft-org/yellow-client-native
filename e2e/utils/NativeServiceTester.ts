/**
 * Utility class for testing the native JavaScript isolate service
 * Since we can't directly access the QuickJS context, we test through:
 * 1. Android logs (Log.d output from Kotlin)
 * 2. WebView bridge calls
 * 3. Network traffic (WebSocket messages)
 */
export class NativeServiceTester {
  private driver: WebdriverIO.Browser;
  
  constructor(driver: WebdriverIO.Browser) {
    this.driver = driver;
  }

  /**
   * Check Android logs for service initialization
   */
  async waitForServiceInitialization(accountId: string, timeout = 10000): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      const logs = await this.driver.getLogs('logcat');
      const serviceInitLog = logs.find(log => 
        log.message.includes('JavaScript context initialized') &&
        log.message.includes(accountId)
      );
      
      if (serviceInitLog) {
        console.log('✅ Service initialized:', serviceInitLog.message);
        return true;
      }
      
      await this.driver.pause(500);
    }
    
    console.log('❌ Service initialization timeout');
    return false;
  }

  /**
   * Check if service file was loaded successfully
   */
  async waitForServiceFileLoaded(timeout = 5000): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      const logs = await this.driver.getLogs('logcat');
      const serviceLoadLog = logs.find(log => 
        log.message.includes('Service loaded:') && 
        !log.message.includes('0 characters')
      );
      
      if (serviceLoadLog) {
        console.log('✅ Service file loaded:', serviceLoadLog.message);
        return true;
      }
      
      await this.driver.pause(500);
    }
    
    return false;
  }

  /**
   * Check for TAURI_MOBILE flag injection
   */
  async waitForTauriMobileInjection(timeout = 5000): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      const logs = await this.driver.getLogs('logcat');
      const tauriMobileLog = logs.find(log => 
        log.message.includes('TAURI_MOBILE') || 
        log.message.includes('globalThis.TAURI_MOBILE = true')
      );
      
      if (tauriMobileLog) {
        console.log('✅ TAURI_MOBILE injected');
        return true;
      }
      
      await this.driver.pause(500);
    }
    
    return false;
  }

  /**
   * Trigger a test message through the WebView to test the bridge
   */
  async sendTestMessageThroughWebView(testMessage: any): Promise<void> {
    // Switch to WebView context
    const contexts = await this.driver.getContexts();
    const webviewContext = contexts.find(ctx => ctx.includes('WEBVIEW'));
    
    if (!webviewContext) {
      throw new Error('No WebView context found');
    }
    
    await this.driver.switchContext(webviewContext);
    
    // Execute JavaScript in WebView to send test message
    await this.driver.executeScript(`
      // Use the Tauri invoke API to send test message
      if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke('plugin:yellow|send_to_native', {
          accountId: 'test-account',
          message: ${JSON.stringify(testMessage)}
        }).catch(console.error);
      }
    `);
    
    // Switch back to native context
    await this.driver.switchContext('NATIVE_APP');
  }

  /**
   * Wait for service to handle a message (check logs)
   */
  async waitForServiceMessageHandling(messageType: string, timeout = 5000): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      const logs = await this.driver.getLogs('logcat');
      const messageHandledLog = logs.find(log => 
        log.message.includes('[MessagesService]') &&
        log.message.includes(messageType)
      );
      
      if (messageHandledLog) {
        console.log('✅ Service handled message:', messageHandledLog.message);
        return true;
      }
      
      await this.driver.pause(500);
    }
    
    return false;
  }

  /**
   * Check for WebSocket message sending from service
   */
  async waitForWebSocketMessage(expectedCommand: string, timeout = 5000): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      const logs = await this.driver.getLogs('logcat');
      const wsMessageLog = logs.find(log => 
        log.message.includes('sendMessageFromJS') &&
        log.message.includes(expectedCommand)
      );
      
      if (wsMessageLog) {
        console.log('✅ WebSocket message sent:', wsMessageLog.message);
        return true;
      }
      
      await this.driver.pause(500);
    }
    
    return false;
  }

  /**
   * Get all relevant logs for debugging
   */
  async getServiceLogs(): Promise<string[]> {
    const logs = await this.driver.getLogs('logcat');
    return logs
      .filter(log => 
        log.message.includes('Account') ||
        log.message.includes('YellowPlugin') ||
        log.message.includes('MessagesService') ||
        log.message.includes('QuickJS')
      )
      .map(log => `[${log.timestamp}] ${log.level}: ${log.message}`);
  }

  /**
   * Reset service state by restarting the account
   */
  async resetServiceState(): Promise<void> {
    // This would trigger account disconnect/reconnect in the app
    // Implementation depends on how account management is exposed in UI
    console.log('🔄 Resetting service state...');
  }
}