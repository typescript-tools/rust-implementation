// const fs = require('fs')
// const path = require('path')
const getBinary = require('./get-binary')

// Link the downloaded binary into the node_modules/.bin directory
// const linkBinaryIntoBin = () => {
//   const binaryName = 'monorepo'
//   const existingPath = path.resolve(__dirname, `../node_modules/binary-install/bin/${binaryName}`)
//   const desiredPath = path.resolve(__dirname, `../node_modules/.bin/${binaryName}`)
//
//   // Only symlink the path when it does not yet exist
//   if (!fs.existsSync(desiredPath)) {
//     fs.linkSync(existingPath, desiredPath)
//   }
// }

// Downloaded binary to /node_modules/binary-install/bin/{ name }
getBinary().install()
  // .then(linkBinaryIntoBin)
