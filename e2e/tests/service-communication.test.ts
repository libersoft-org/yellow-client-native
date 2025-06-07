import { NativeServiceTester } from '../utils/NativeServiceTester.js';

describe('Native Service Communication', () => {
  let serviceTester: NativeServiceTester;

  beforeEach(async () => {
    serviceTester = new NativeServiceTester(global.driver);
    
    // Ensure service is initialized before communication tests
    const serviceReady = await serviceTester.waitForServiceInitialization('test-account');
    expect(serviceReady).toBe(true);
  });

  test('should inject send() function and route to WebSocket', async () => {
    // Test that send() function is available and routes messages correctly
    await serviceTester.sendTestMessageThroughWebView({
      target: 'org.libersoft.messages',
      command: 'test_send_function',
      params: { test: true }
    });

    // Check that the message was processed by the send() function
    const messageProcessed = await serviceTester.waitForWebSocketMessage('test_send_function');
    expect(messageProcessed).toBe(true);
  });

  test('should route WebSocket messages to service', async () => {
    // Simulate a WebSocket message coming from server
    // This would normally be done by triggering server-side events
    // For testing, we can use the WebView to simulate the message flow
    
    await serviceTester.sendTestMessageThroughWebView({
      module: 'org.libersoft.messages',
      event: 'new_message',
      data: {
        message: 'Test message from server',
        address_from: 'test@example.com',
        address_to: 'user@example.com'
      }
    });

    // Check that service received and processed the message
    const messageHandled = await serviceTester.waitForServiceMessageHandling('new_message');
    expect(messageHandled).toBe(true);
  });

  test('should handle service-to-native bridge communication', async () => {
    // Test that service can send messages back through KotlinBridge
    await serviceTester.sendTestMessageThroughWebView({
      module: 'org.libersoft.messages',
      event: 'test_bridge_communication',
      data: { triggerResponse: true }
    });

    // Service should process this and send a response through KotlinBridge
    const bridgeResponseSent = await serviceTester.waitForWebSocketMessage('bridge_response');
    expect(bridgeResponseSent).toBe(true);
  });

  test('should handle subscription management', async () => {
    // Test that service properly subscribes to events when TAURI_MOBILE=true
    const logs = await serviceTester.getServiceLogs();
    
    // Should see subscription initialization
    const hasSubscriptions = logs.some(log => 
      log.includes('Subscriptions initialized: yes') ||
      log.includes('subscribe') && log.includes('new_message')
    );
    
    expect(hasSubscriptions).toBe(true);
  });

  test('should handle service errors gracefully', async () => {
    // Send malformed message to test error handling
    await serviceTester.sendTestMessageThroughWebView({
      module: 'org.libersoft.messages',
      event: 'invalid_event',
      data: { malformed: 'data' }
    });

    // Service should handle this gracefully and log the error
    const errorHandled = await serviceTester.waitForServiceMessageHandling('invalid_event');
    // Should either handle it or show appropriate error logging
    const logs = await serviceTester.getServiceLogs();
    const hasErrorHandling = logs.some(log => 
      log.includes('Unhandled message event') || 
      log.includes('invalid_event')
    );
    
    expect(hasErrorHandling).toBe(true);
  });

  afterEach(async () => {
    // Reset service state between tests
    await serviceTester.resetServiceState();
  });
});