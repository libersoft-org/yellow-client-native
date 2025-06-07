# Yellow Native Android E2E Tests

This directory contains end-to-end tests for the Yellow native Android application, specifically testing the integration between the WebView frontend, native Kotlin layer, and QuickJS JavaScript isolate service.

## Architecture Under Test

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   WebView       │    │   Native Kotlin  │    │  QuickJS Isolate│
│   (Frontend)    │◄──►│   (Account.kt)   │◄──►│  (Service.js)   │
│   - Svelte UI   │    │   - WebSocket    │    │   - Messages    │
│   - Tauri APIs  │    │   - JS Context   │    │   - Background  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Testing Strategy

Since the service runs in a native JavaScript isolate (not in the WebView), we test through:

1. **Android Logs** - Monitor Kotlin Log.d output for service lifecycle
2. **WebView Bridge** - Test communication through Tauri's `send_to_native` API
3. **Network Traffic** - Verify WebSocket message routing

## Prerequisites

### 1. Install Dependencies
```bash
cd e2e
npm install
npm run appium:install
```

### 2. Setup Android Environment
- Android SDK with API level 30+
- Android emulator or physical device
- USB debugging enabled

### 3. Build Yellow App
```bash
# Build the service file
cd ../
bun run build:messages-service

# Build Android APK
bun run tauri build --target android
```

### 4. Start Appium Server
```bash
npm run appium:start
```

## Running Tests

### All Tests
```bash
npm run test:android
```

### Specific Test Suites
```bash
# Service initialization only
npx jest service-initialization.test.ts

# Communication tests
npx jest service-communication.test.ts

# Full integration flow
npx jest integration-flow.test.ts
```

## Test Structure

### `service-initialization.test.ts`
- Service file loading from assets/dev server
- TAURI_MOBILE flag injection
- YellowMessagesService initialization
- Fallback service handling

### `service-communication.test.ts`
- send() function injection and WebSocket routing
- Message routing from WebSocket to service
- KotlinBridge communication
- Subscription management
- Error handling

### `integration-flow.test.ts`
- Complete message flows in both directions
- Event subscription and handling
- Error recovery and resilience testing

## Debugging

### View Service Logs
```bash
# Filter for relevant logs during test run
adb logcat | grep -E "(Account|YellowPlugin|MessagesService)"
```

### Common Issues

1. **Service file not found**
   - Ensure `bun run build:messages-service` was run
   - Check APK includes `assets/tmp/messages-service.js`

2. **Appium connection issues**
   - Verify emulator is running: `adb devices`
   - Check Appium server is on port 4723
   - Update `appium.config.ts` with correct device/package names

3. **WebView context not found**
   - App may not have loaded completely
   - Check if WebView debugging is enabled
   - Verify correct package name in config

## Configuration

### `appium.config.ts`
Update the following for your environment:
- `deviceName` - Your emulator/device ID
- `app` - Path to built APK
- `appPackage` - Your app's package name
- `chromedriverExecutable` - Path to chromedriver

### Test Timeouts
Default timeouts are set for emulator performance:
- Service initialization: 10 seconds
- Message processing: 5 seconds
- Overall test timeout: 60 seconds

Adjust in the test files if needed for slower devices.

## CI/CD Integration

For GitHub Actions or other CI systems:

```yaml
- name: Run E2E Tests
  run: |
    cd yellow-client-native/e2e
    npm install
    npm run test:android
```

Ensure the CI environment has:
- Android SDK
- Emulator setup
- Appium server running