const fs = require('fs')
const path = require('path')
const tmp = require('tmp')
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
  const tempPath = tmp.tmpNameSync({ tmpdir: nodeModulesBinDir })
  const existingPath = path.resolve(downloadDir, binaryName)
  const desiredPath = path.resolve(nodeModulesBinDir, binaryName)

  // First, link the binary with a temporary file. If this fails and throw an error,
  // then we'll end up doing nothing. This uses a hard link to avoid taking up
  // additional space on the file system.
  fs.linkSync(existingPath, tempPath)

  // Then use rename to atomically replace the target file with the temporary file.
  // If this fails and throws an error, we'll end up leaving the temporary file there,
  // which is harmless.
  fs.renameSync(tempPath, desiredPath)
}

// Downloaded binary to /bin/{ name }
getBinary().install()
  .then(linkBinaryIntoBin)
