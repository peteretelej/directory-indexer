#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

function getBinaryPath() {
  const platform = process.platform;
  const arch = process.arch;
  
  let binaryName = 'directory-indexer';
  let platformDir = '';
  
  if (platform === 'win32') {
    binaryName += '.exe';
    platformDir = 'win32-x64';
  } else if (platform === 'darwin') {
    platformDir = arch === 'arm64' ? 'darwin-arm64' : 'darwin-x64';
  } else if (platform === 'linux') {
    platformDir = 'linux-x64';
  } else {
    console.error(`Unsupported platform: ${platform}-${arch}`);
    process.exit(1);
  }
  
  // Try different locations for the binary
  const possiblePaths = [
    // Pre-built binary from npm package
    path.join(__dirname, '..', 'binaries', `${binaryName}-${platformDir}`),
    // Development build
    path.join(__dirname, '..', 'target', 'release', binaryName),
    path.join(__dirname, '..', 'target', 'debug', binaryName),
  ];
  
  for (const binaryPath of possiblePaths) {
    if (fs.existsSync(binaryPath)) {
      return binaryPath;
    }
  }
  
  console.error(`Binary not found for ${platform}-${arch}`);
  console.error('Tried paths:', possiblePaths);
  console.error('Run "cargo build --release" to build the binary');
  process.exit(1);
}

function main() {
  const binaryPath = getBinaryPath();
  const args = process.argv.slice(2);
  
  const child = spawn(binaryPath, args, {
    stdio: 'inherit',
    windowsHide: false,
  });
  
  child.on('error', (error) => {
    console.error('Failed to start directory-indexer:', error.message);
    process.exit(1);
  });
  
  child.on('close', (code) => {
    process.exit(code);
  });
}

if (require.main === module) {
  main();
}

module.exports = { getBinaryPath };