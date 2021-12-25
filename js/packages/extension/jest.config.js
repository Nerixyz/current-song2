/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testRegex: 'test/.*\\.test\\.[jt]s$',
  coveragePathIgnorePatterns: ['test'],
};
