module.exports = {
  testMatch: [
    '**/__tests__/**/*test.bs.js'
  ],
  transform: {
    '^.+\\.js$': './jest-transform.js',
  },
  transformIgnorePatterns: [
    // transform ES6 modules generated by BuckleScript
    // https://regexr.com/46984
    '/node_modules/(?!(@.*/)?(bs-.*|reason-.*)/).+\\.js$',
  ],
  watchPathIgnorePatterns: ['/node_modules'],
};
