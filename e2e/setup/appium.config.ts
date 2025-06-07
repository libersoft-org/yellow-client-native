export const appiumConfig = {
  port: 4723,
  capabilities: {
    platformName: 'Android',
    'appium:platformVersion': '11.0', // Adjust based on your test device
    'appium:deviceName': 'emulator-5554',
    'appium:app': './src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk',
    'appium:appPackage': 'org.libersoft.yellow', // Update with actual package name
    'appium:appActivity': '.MainActivity',
    'appium:automationName': 'UiAutomator2',
    'appium:noReset': false,
    'appium:fullReset': true,
    'appium:newCommandTimeout': 300,
    // Enable WebView testing
    'appium:chromedriverExecutable': '/usr/local/bin/chromedriver', // Update path as needed
    'appium:autoWebview': false, // We'll manually switch contexts
    // Enable logging
    'appium:enablePerformanceLogging': true,
    'appium:logcatFormat': 'process',
    'appium:logcatFilterSpecs': [
      'Account:V', // Our Kotlin logging
      'YellowPlugin:V',
      'chromium:V' // WebView logs
    ]
  }
};