const fs = require('fs')
const path = require('path')
const getBinary = require('./get-binary')

const binaryName = 'monorepo'
const repoDir = path.dirname(__dirname)
const downloadDir = path.resolve(repoDir, 'bin')
const scopedPackageDir = path.dirname(repoDir)
const nodeModulesDir = path.dirname(scopedPackageDir)
const nodeModulesBinDir = path.resolve(nodeModulesDir, '.bin')

// Link the downloaded binary into the node_modules/.bin directory.
// Inspiration from https://github.com/evanw/esbuild/releases/tag/v0.13.4
const linkBinaryIntoBin = () => {
  const existingPath = path.resolve(downloadDir, binaryName)
  const desiredPath = path.resolve(nodeModulesBinDir, binaryName)

  // Remove the symlinked, dummy, script, before symlinking the compiled binary in place
  fs.unlinkSync(desiredPath)
  fs.symlinkSync(existingPath, desiredPath)

  // By default, the downloaded file preserves the mtime of the binary created by GitHub
  // Actions. For a seamless GNU Make experience on local machines, we want to update the
  // mtime of this binary to the time of download.
  const now = new Date()
  fs.utimesSync(desiredPath, now, now)
  fs.utimesSync(existingPath, now, now)
}

// Downloaded binary to /bin/{ name }
getBinary().install()
  .then(linkBinaryIntoBin)
