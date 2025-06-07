import { remote } from 'webdriverio';
import type { RemoteOptions } from 'webdriverio';

declare global {
  var driver: WebdriverIO.Browser;
}

export default async function globalSetup() {
  console.log('🚀 Starting Appium E2E test setup...');
  
  try {
    // Create driver instance
    const options: RemoteOptions = {
      hostname: 'localhost',
      port: 4723,
      path: '/',
      capabilities: {
        'platformName': 'Android',
        'appium:platformVersion': '11.0',
        'appium:deviceName': 'emulator-5554',
        'appium:app': './src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk',
        'appium:appPackage': 'org.libersoft.yellow',
        'appium:appActivity': '.MainActivity',
        'appium:automationName': 'UiAutomator2',
        'appium:noReset': false,
        'appium:fullReset': true,
        'appium:newCommandTimeout': 300
      },
      logLevel: 'info'
    };
    
    global.driver = await remote(options);

    console.log('✅ Appium driver connected');
    
    // Wait for app to launch
    await global.driver.pause(3000);
    
    console.log('📱 App launched successfully');
    
  } catch (error) {
    console.error('❌ Failed to setup Appium:', error);
    throw error;
  }
}