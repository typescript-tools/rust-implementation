// When installing, the `preinstall` script is executed before any dependencies are
// installed. Since this script `require`s a dependency, another developer who clones
// the repository and runs `npm install` will get an error that dependencies are
// missing.
//
// We guard against this with a try/catch to swallow the error: if the dependencies are
// not found it means the package wasn't installed yet, and that means there's no binary
// to uninstall in the first place.

const getBinary = () => {
  try {
    const getBinary = require('./get-binary')
    return getBinary()
  } catch {

  }
}

const binary = getBinary()
console.log("Binary is", binary)
if (binary) {
  binary.uninstall()
}
