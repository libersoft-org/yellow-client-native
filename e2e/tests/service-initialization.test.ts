import { NativeServiceTester } from '../utils/NativeServiceTester.js';

describe('Native JavaScript Service Initialization', () => {
  let serviceTester: NativeServiceTester;

  beforeEach(() => {
    serviceTester = new NativeServiceTester(global.driver);
  });

  test('should load service file successfully', async () => {
    // Check that service file is loaded from static assets or dev server
    const serviceLoaded = await serviceTester.waitForServiceFileLoaded();
    expect(serviceLoaded).toBe(true);
  });

  test('should inject TAURI_MOBILE flag', async () => {
    // Verify that TAURI_MOBILE=true is set in the JS context
    const tauriMobileInjected = await serviceTester.waitForTauriMobileInjection();
    expect(tauriMobileInjected).toBe(true);
  });

  test('should initialize YellowMessagesService', async () => {
    // Test that the service initializes with proper account config
    const serviceInitialized = await serviceTester.waitForServiceInitialization('test-account');
    expect(serviceInitialized).toBe(true);
  });

  test('should handle service initialization failure gracefully', async () => {
    // Test fallback service when main service fails to load
    const logs = await serviceTester.getServiceLogs();
    const hasFallback = logs.some(log => 
      log.includes('using fallback') || 
      log.includes('Fallback service')
    );
    
    // Should either succeed normally or fail gracefully with fallback
    expect(hasFallback || logs.some(log => log.includes('Service loaded:'))).toBe(true);
  });

  afterEach(async () => {
    // Log debugging info if test failed
    if (expect.getState().currentTestName?.includes('should')) {
      const logs = await serviceTester.getServiceLogs();
      console.log('📋 Service logs:', logs.slice(-10)); // Last 10 logs
    }
  });
});