/** @type{import('jest').Config} */
export default {
  coverageProvider: 'v8',
  coverageDirectory: 'coverage',
  setupFilesAfterEnv: ['<rootDir>/tests/setup.ts'],
  moduleNameMapper: {
    '^#/(.*)$': '<rootDir>/src/$1',
    '^.+\\.(svg)$': '<rootDir>/tests/__mocks__/svg.ts'
  },
  transform: {
    '^.+\\.(t|j)sx?$': '@swc/jest'
  },
  testEnvironment: 'jsdom',
  collectCoverageFrom: ['src/**/*.{ts,tsx}', '!src/**/*.d.ts'],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  }
}
