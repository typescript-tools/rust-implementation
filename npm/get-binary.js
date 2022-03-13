const os = require('os')
const { Binary } = require('./binary-install')

const knownUnixLikePackages = {
  'linux x64 LE': 'x86_64-unknown-linux-musl',
  'linux ia32 LE': 'i686-unknown-linux-musl',
  'darwin x64 LE': 'x86_64-apple-darwin',
  'linux arm64 LE': 'aarch64-unknown-linux-musl',
  'darwin arm64 LE': 'aarch64-apple-darwin',
}

const binPathForCurrentPlatform = () => {
  const platformKey = `${process.platform} ${os.arch()} ${os.endianness()}`
  if (platformKey in knownUnixLikePackages) {
    return knownUnixLikePackages[platformKey]
  }

  throw new Error('Unsupported platform: ${platformKey}')
}

const getBinary = () => {
  const { version } = require('../package.json')
  const platform = binPathForCurrentPlatform()
  const url = `https://github.com/typescript-tools/rust-implementation/releases/download/v${version}/typescript-tools-${platform}.tar.gz`
  const binaryName = 'monorepo'
  return new Binary(binaryName, url)
}

module.exports = getBinary
