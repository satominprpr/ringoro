const withTM = require('next-transpile-modules')(['bs-platform'])

module.exports = withTM({
  pageExtensions: ['jsx', 'js', 'bs.js'],
})

module.exports.webpack = function (config) {
  config.module.rules.push(
    {
      test: /\.ya?ml$/,
      use: 'js-yaml-loader',
    },
  )

  return config
}
