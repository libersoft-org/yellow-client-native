export default {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['<rootDir>/tests/**/*.test.ts'],
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      useESM: true
    }]
  },
  extensionsToTreatAsEsm: ['.ts'],
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1'
  },
  setupFilesAfterEnv: ['<rootDir>/setup/jest.setup.ts'],
  testTimeout: 60000, // 60 seconds for mobile tests
  globalSetup: '<rootDir>/setup/global.setup.ts',
  globalTeardown: '<rootDir>/setup/global.teardown.ts'
};