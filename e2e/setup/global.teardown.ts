export default async function globalTeardown() {
  console.log('🧹 Cleaning up Appium E2E tests...');
  
  try {
    if (global.driver) {
      await global.driver.deleteSession();
      console.log('✅ Appium session closed');
    }
  } catch (error) {
    console.error('❌ Error during teardown:', error);
  }
}