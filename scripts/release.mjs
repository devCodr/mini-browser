#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const pkgJsonPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

const args = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const typeOrVersion = args[0] || 'patch';
const skipBuild = process.argv.includes('--no-build');
const skipGit = process.argv.includes('--no-git');

// 1. Read current version
const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
const currentVersion = pkg.version || '1.0.0';
console.log(`Current version: ${currentVersion}`);

// 2. Compute new version
function bump(curr, type) {
  const parts = curr.split('.').map((n) => parseInt(n, 10) || 0);
  while (parts.length < 3) parts.push(0);
  if (type === 'major') {
    return `${parts[0] + 1}.0.0`;
  } else if (type === 'minor') {
    return `${parts[0]}.${parts[1] + 1}.0`;
  } else if (type === 'patch') {
    return `${parts[0]}.${parts[1]}.${parts[2] + 1}`;
  } else if (/^\d+\.\d+\.\d+/.test(type)) {
    return type;
  }
  throw new Error(`Invalid bump type or version: ${type}`);
}

const newVersion = bump(currentVersion, typeOrVersion);
console.log(`Bumping version to: ${newVersion}`);

// 3. Update package.json
pkg.version = newVersion;
fs.writeFileSync(pkgJsonPath, JSON.stringify(pkg, null, 2) + '\n');
console.log(`✓ Updated ${path.relative(rootDir, pkgJsonPath)}`);

// 4. Update tauri.conf.json
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = newVersion;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
console.log(`✓ Updated ${path.relative(rootDir, tauriConfPath)}`);

// 5. Update Cargo.toml
let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
cargoToml = cargoToml.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
fs.writeFileSync(cargoTomlPath, cargoToml);
console.log(`✓ Updated ${path.relative(rootDir, cargoTomlPath)}`);

// Update Cargo.lock
try {
  execSync('cargo check --quiet', { stdio: 'inherit', cwd: path.join(rootDir, 'src-tauri') });
} catch (_) {}

// 6. Git commit & tag
if (!skipGit) {
  try {
    execSync(`git add "${pkgJsonPath}" "${tauriConfPath}" "${cargoTomlPath}"`, { stdio: 'inherit', cwd: rootDir });
    execSync('git add -A', { stdio: 'inherit', cwd: rootDir });
    execSync(`git commit -m "chore(release): v${newVersion}"`, { stdio: 'inherit', cwd: rootDir });
    console.log(`✓ Committed release: chore(release): v${newVersion}`);
  } catch (err) {
    console.log('No new uncommitted changes, proceeding to tag...');
  }

  try {
    execSync(`git tag -a "v${newVersion}" -m "Release v${newVersion}"`, { stdio: 'inherit', cwd: rootDir });
    console.log(`✓ Created git tag: v${newVersion}`);
  } catch (err) {
    console.warn(`Tag v${newVersion} might already exist or git tag failed:`, err.message);
  }
}

// 7. Compile release build
if (!skipBuild) {
  console.log(`\nCompiling native release binary & bundles for v${newVersion}...`);
  execSync('pnpm run build', { stdio: 'inherit', cwd: rootDir });
  console.log(`\n🎉 Release v${newVersion} built successfully!`);
}
