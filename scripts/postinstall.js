#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function makeExecutable(filePath) {
  if (fs.existsSync(filePath)) {
    try {
      fs.chmodSync(filePath, 0o755);
      console.log(`Made ${filePath} executable`);
    } catch (error) {
      console.warn(`Failed to make ${filePath} executable:`, error.message);
    }
  }
}

function main() {
  const platform = process.platform;
  const arch = process.arch;
  
  // Make the wrapper script executable
  const wrapperPath = path.join(__dirname, '..', 'bin', 'directory-indexer.js');
  makeExecutable(wrapperPath);
  
  // Make the platform-specific binary executable
  let binaryName, platformSuffix;
  
  if (platform === 'win32') {
    binaryName = 'directory-indexer.exe-win32-x64';
  } else if (platform === 'darwin') {
    binaryName = arch === 'arm64' ? 'directory-indexer-darwin-arm64' : 'directory-indexer-darwin-x64';
  } else if (platform === 'linux') {
    binaryName = 'directory-indexer-linux-x64';
  }
  
  if (binaryName) {
    const binaryPath = path.join(__dirname, '..', 'binaries', binaryName);
    makeExecutable(binaryPath);
  }
  
  console.log('directory-indexer installed successfully!');
  console.log('Run "directory-indexer --help" to get started.');
}

if (require.main === module) {
  main();
}