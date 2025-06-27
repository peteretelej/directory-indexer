#!/usr/bin/env node

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const targets = [
  { target: 'x86_64-unknown-linux-gnu', platform: 'linux', arch: 'x64' },
  { target: 'x86_64-apple-darwin', platform: 'darwin', arch: 'x64' },
  { target: 'aarch64-apple-darwin', platform: 'darwin', arch: 'arm64' },
  { target: 'x86_64-pc-windows-gnu', platform: 'win32', arch: 'x64' },
];

function runCommand(command, args = [], options = {}) {
  return new Promise((resolve, reject) => {
    console.log(`Running: ${command} ${args.join(' ')}`);
    const child = spawn(command, args, { stdio: 'inherit', ...options });
    
    child.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command failed with exit code ${code}`));
      }
    });
    
    child.on('error', reject);
  });
}

async function buildTarget(targetInfo) {
  const { target, platform, arch } = targetInfo;
  
  console.log(`\n🏗️  Building for ${platform}-${arch} (${target})`);
  
  try {
    // Add the target if not already installed
    await runCommand('rustup', ['target', 'add', target]);
    
    // Build for the target
    await runCommand('cargo', ['build', '--release', '--target', target]);
    
    // Copy binary to binaries directory
    const binariesDir = path.join(__dirname, '..', 'binaries');
    if (!fs.existsSync(binariesDir)) {
      fs.mkdirSync(binariesDir, { recursive: true });
    }
    
    const extension = platform === 'win32' ? '.exe' : '';
    const sourcePath = path.join(__dirname, '..', 'target', target, 'release', `directory-indexer${extension}`);
    const destPath = path.join(binariesDir, `directory-indexer-${platform}-${arch}${extension}`);
    
    if (fs.existsSync(sourcePath)) {
      fs.copyFileSync(sourcePath, destPath);
      fs.chmodSync(destPath, 0o755);
      console.log(`✅ Built ${destPath}`);
    } else {
      throw new Error(`Binary not found at ${sourcePath}`);
    }
    
  } catch (error) {
    console.error(`❌ Failed to build ${platform}-${arch}:`, error.message);
    throw error;
  }
}

async function main() {
  console.log('🚀 Building directory-indexer for all platforms...\n');
  
  // Build for all targets
  for (const target of targets) {
    try {
      await buildTarget(target);
    } catch (error) {
      console.error(`Failed to build ${target.platform}-${target.arch}, continuing...`);
    }
  }
  
  console.log('\n✨ Build complete! Binaries are in the binaries/ directory.');
}

if (require.main === module) {
  main().catch((error) => {
    console.error('Build failed:', error);
    process.exit(1);
  });
}