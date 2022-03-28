const { createHash } = require("crypto");
const { chmodSync, existsSync, mkdirSync, readFileSync } = require("fs");
const { join } = require("path");
const { spawnSync } = require("child_process");

const axios = require("axios");
const tar = require("tar");
const rimraf = require("rimraf");

class Binary {
  constructor(name, url) {
    let errors = [];
    if (typeof url !== "string") {
      errors.push("url must be a string");
    } else {
      try {
        new URL(url);
      } catch (e) {
        errors.push(e);
      }
    }
    if (name && typeof name !== "string") {
      errors.push("name must be a string");
    }

    if (!name) {
      errors.push("You must specify the name of your binary");
    }
    if (errors.length > 0) {
      let errorMsg =
        "One or more of the parameters you passed to the Binary constructor are invalid:\n";
      errors.forEach(error => {
        errorMsg += error;
      });
      errorMsg +=
        '\n\nCorrect usage: new Binary("my-binary", "https://example.com/binary/download.tar.gz")';
      console.error(errorMsg);
      process.exit(1);
    }
    this.url = url;
    this.name = name;
    this.installDirectory = join(__dirname,  "../bin");

    if (!existsSync(this.installDirectory)) {
      mkdirSync(this.installDirectory, { recursive: true });
    }

    this.binaryPath = join(this.installDirectory, this.name);
  }

  install(fetchOptions) {
    if (existsSync(this.installDirectory)) {
      rimraf.sync(this.installDirectory);
    }

    mkdirSync(this.installDirectory, { recursive: true });

    // console.log(`Downloading release from ${this.url}`);

    // Stream the file from the GitHub release page
    return axios({ ...fetchOptions, url: this.url, responseType: "stream" })
      // untar the stream and write to disk
      .then(res => {
        return new Promise((resolve, reject) => {
          const sink = tar.x({ strip: 1, C: this.installDirectory });
          res.data.pipe(sink);
          sink.on('finish', () => resolve());
          sink.on('error', err => reject(err));
        });
      })
      // calculate a checksum of the untarred binary
      .then(() => {
        const fileBuffer = readFileSync(this.binaryPath);
        const hashsum = createHash("sha256");
        hashsum.update(fileBuffer);
        const calculated_checksum = hashsum.digest('hex');

        const advertised_checksums = readFileSync(join(__dirname, "SHASUMS256.txt"));

        return new Promise((resolve, reject) => {
          if (advertised_checksums.includes(calculated_checksum)) {
            resolve();
          } else {
            chmodSync(this.binaryPath, 0o400);
            console.error(`Calculated unexpected checksum ${calculated_checksum} for file ${this.binaryPath}`);
            console.error('This file has been stripped of executable permissions but you should quarantine or delete it and open an issue.');
            reject(new Error('Unexpected checksum'));
          }
        });
      })
      .then(() => {
        // console.log(`${this.name} has been installed!`);
      })
      .catch(err => {
        console.error(`Error fetching release:`);
        console.error(err);
        process.exit(1);
      });
  }

  run() {
    if (!existsSync(this.binaryPath)) {
      console.error(`You must install ${this.name} before you can run it`);
      process.exit(1);
    }

    const [, , ...args] = process.argv;

    const options = { cwd: process.cwd(), stdio: "inherit" };

    const result = spawnSync(this.binaryPath, args, options);

    if (result.error) {
      console.error(result.error);
      process.exit(1);
    }

    process.exit(result.status);
  }
}

module.exports.Binary = Binary;
