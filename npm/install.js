const fs = require('fs')
const path = require('path')
const getBinary = require('./get-binary')

const binaryName = 'monorepo'
const repoDir = path.dirname(__dirname)
const downloadDir = path.resolve(repoDir, 'bin')
const scopedPackageDir = path.dirname(repoDir)
const nodeModulesDir = path.dirname(scopedPackageDir)

// Link the downloaded binary into the node_modules/.bin directory
const linkBinaryIntoBin = () => {
  const existingPath = path.resolve(downloadDir, binaryName)
  const desiredPath = path.resolve(nodeModulesDir, '.bin', binaryName)

  // Only symlink the path when it does not yet exist
  if (!fs.existsSync(desiredPath)) {
    fs.linkSync(existingPath, desiredPath)
  }
}

// Downloaded binary to /bin/{ name }
getBinary().install()
  .then(linkBinaryIntoBin)
