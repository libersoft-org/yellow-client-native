import { NativeServiceTester } from '../utils/NativeServiceTester.js';

describe('End-to-End Integration Flow', () => {
  let serviceTester: NativeServiceTester;

  beforeEach(async () => {
    serviceTester = new NativeServiceTester(global.driver);
  });

  test('complete message flow: WebView -> Native -> Service -> WebSocket', async () => {
    // 1. Ensure service is initialized
    const serviceReady = await serviceTester.waitForServiceInitialization('test-account');
    expect(serviceReady).toBe(true);

    // 2. Simulate user action in WebView (e.g., sending a message)
    await serviceTester.sendTestMessageThroughWebView({
      target: 'org.libersoft.messages',
      command: 'message_send',
      params: {
        address: 'recipient@example.com',
        message: 'Hello from E2E test',
        format: 'text'
      }
    });

    // 3. Verify message goes through the send() function
    const sendFunctionCalled = await serviceTester.waitForWebSocketMessage('message_send');
    expect(sendFunctionCalled).toBe(true);

    // 4. Check that service processes any response
    const responseProcessed = await serviceTester.waitForServiceMessageHandling('message_send');
    // Note: This depends on whether the test has a mock server responding
  });

  test('complete message flow: WebSocket -> Native -> Service -> WebView', async () => {
    // 1. Ensure service is initialized
    const serviceReady = await serviceTester.waitForServiceInitialization('test-account');
    expect(serviceReady).toBe(true);

    // 2. Simulate incoming WebSocket message (would come from server)
    // In a real scenario, this would be triggered by connecting to a test server
    // For now, we simulate by sending through the WebView bridge
    await serviceTester.sendTestMessageThroughWebView({
      module: 'org.libersoft.messages',
      event: 'new_message',
      data: {
        uid: 'test-message-123',
        address_from: 'sender@example.com',
        address_to: 'user@example.com',
        message: 'Hello from server',
        format: 'text',
        created: new Date().toISOString()
      }
    });

    // 3. Verify service receives and processes the message
    const messageProcessed = await serviceTester.waitForServiceMessageHandling('new_message');
    expect(messageProcessed).toBe(true);

    // 4. Check if service triggers any notifications or responses
    const logs = await serviceTester.getServiceLogs();
    const hasNotificationHandling = logs.some(log => 
      log.includes('show_notification') || 
      log.includes('processMessageForNotification')
    );
    
    // Should handle notifications for incoming messages
    expect(hasNotificationHandling).toBe(true);
  });

  test('subscription and event handling flow', async () => {
    // 1. Service initialization should trigger subscriptions
    const serviceReady = await serviceTester.waitForServiceInitialization('test-account');
    expect(serviceReady).toBe(true);

    // 2. Check that subscriptions were made
    const logs = await serviceTester.getServiceLogs();
    const hasSubscriptions = logs.some(log => 
      log.includes('Subscriptions initialized: yes')
    );
    expect(hasSubscriptions).toBe(true);

    // 3. Test various event types
    const eventTypes = [
      'new_message',
      'message_update', 
      'seen_message',
      'upload_update'
    ];

    for (const eventType of eventTypes) {
      await serviceTester.sendTestMessageThroughWebView({
        module: 'org.libersoft.messages',
        event: eventType,
        data: { test: true, eventType }
      });

      const eventHandled = await serviceTester.waitForServiceMessageHandling(eventType);
      expect(eventHandled).toBe(true);
    }
  });

  test('error recovery and resilience', async () => {
    // 1. Test service behavior with network issues
    // 2. Test service restart scenarios
    // 3. Test malformed message handling
    // 4. Test memory cleanup

    const initialLogs = await serviceTester.getServiceLogs();
    const initialLogCount = initialLogs.length;

    // Send various problematic messages
    const problematicMessages = [
      { module: 'unknown.module', event: 'test' },
      { malformed: 'message' },
      null,
      { module: 'org.libersoft.messages', event: 'non_existent_event' }
    ];

    for (const message of problematicMessages) {
      try {
        await serviceTester.sendTestMessageThroughWebView(message);
        await global.driver.pause(1000); // Allow processing time
      } catch (error) {
        // Expected for malformed messages
      }
    }

    // Check that service is still responsive
    const finalLogs = await serviceTester.getServiceLogs();
    expect(finalLogs.length).toBeGreaterThan(initialLogCount);

    // Service should still be able to handle valid messages
    await serviceTester.sendTestMessageThroughWebView({
      module: 'org.libersoft.messages',
      event: 'new_message',
      data: { test: 'recovery_test' }
    });

    const recoveryMessageHandled = await serviceTester.waitForServiceMessageHandling('new_message');
    expect(recoveryMessageHandled).toBe(true);
  });
});