#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

// Determine platform and architecture
const platform = os.platform();
const arch = os.arch();

let osName;
let archName;

switch (platform) {
  case 'linux':
    osName = 'linux';
    break;
  case 'darwin':
    osName = 'darwin';
    break;
  case 'win32':
    osName = 'windows';
    break;
  default:
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
}

switch (arch) {
  case 'x64':
    archName = 'x86_64';
    break;
  case 'arm64':
    archName = 'aarch64';
    break;
  default:
    console.error(`Unsupported architecture: ${arch}`);
    process.exit(1);
}

const version = require('../package.json').version;
const binaryName = platform === 'win32' ? 'costpilot.exe' : 'costpilot';
const downloadUrl = `https://github.com/Dee66/CostPilot/releases/download/v${version}/costpilot-${osName}-${archName}${platform === 'win32' ? '.exe' : ''}`;
const binPath = path.join(__dirname, '..', 'bin', binaryName);

console.log(`Downloading CostPilot ${version} for ${osName}-${archName}...`);

// Download the binary
const file = fs.createWriteStream(binPath);
const request = https.get(downloadUrl, (response) => {
  if (response.statusCode !== 200) {
    console.error(`Failed to download: ${response.statusCode} ${response.statusMessage}`);
    console.error(`URL: ${downloadUrl}`);
    process.exit(1);
  }

  response.pipe(file);

  file.on('finish', () => {
    file.close();

    // Make executable on Unix-like systems
    if (platform !== 'win32') {
      try {
        fs.chmodSync(binPath, '755');
      } catch (error) {
        console.error(`Failed to make binary executable: ${error.message}`);
        process.exit(1);
      }
    }

    console.log('CostPilot installed successfully!');
    console.log(`Binary location: ${binPath}`);

    // Test the installation
    try {
      const output = execSync(`"${binPath}" --version`, { encoding: 'utf8' });
      console.log(`Version: ${output.trim()}`);
    } catch (error) {
      console.error('Installation verification failed');
      process.exit(1);
    }
  });
});

request.on('error', (error) => {
  console.error(`Download failed: ${error.message}`);
  process.exit(1);
});

file.on('error', (error) => {
  console.error(`File write failed: ${error.message}`);
  process.exit(1);
});