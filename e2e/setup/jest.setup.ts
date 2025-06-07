// Jest setup file - runs before each test file
declare global {
  var driver: WebdriverIO.Browser;
}

// Extend Jest matchers for mobile testing
expect.extend({
  async toBeDisplayedOnDevice(received: WebdriverIO.Element) {
    const isDisplayed = await received.isDisplayed();
    if (isDisplayed) {
      return {
        message: () => `Element is displayed on device`,
        pass: true,
      };
    } else {
      return {
        message: () => `Element is not displayed on device`,
        pass: false,
      };
    }
  },
});