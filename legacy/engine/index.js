const { existsSync } = require('fs');
const { join } = require('path');

const platform = process.platform;
const arch = process.arch;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      return readFileSync('/usr/bin/ldd', 'utf8').includes('musl');
    } catch (e) {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case 'linux':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'pronbrow_engine.linux-x64-gnu.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./pronbrow_engine.linux-x64-gnu.node');
          } else {
            nativeBinding = require('@pronbrow/engine-linux-x64-gnu');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'pronbrow_engine.linux-arm64-gnu.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./pronbrow_engine.linux-arm64-gnu.node');
          } else {
            nativeBinding = require('@pronbrow/engine-linux-arm64-gnu');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  case 'darwin':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'pronbrow_engine.darwin-x64.node'));
        try {
          if (localFileExisted) {
            nativeBinding = require('./pronbrow_engine.darwin-x64.node');
          } else {
            nativeBinding = require('@pronbrow/engine-darwin-x64');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'pronbrow_engine.darwin-arm64.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./pronbrow_engine.darwin-arm64.node');
          } else {
            nativeBinding = require('@pronbrow/engine-darwin-arm64');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'pronbrow_engine.win32-x64-msvc.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./pronbrow_engine.win32-x64-msvc.node');
          } else {
            nativeBinding = require('@pronbrow/engine-win32-x64-msvc');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError;
  }
  throw new Error(`Failed to load native binding`);
}

const { PronBrowEngine, BrowserTab, createBrowserEngine, getVersion } = nativeBinding;

module.exports.PronBrowEngine = PronBrowEngine;
module.exports.BrowserTab = BrowserTab;
module.exports.createBrowserEngine = createBrowserEngine;
module.exports.getVersion = getVersion;
