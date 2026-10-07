import { spawnSync } from 'node:child_process';

const args = [
  'build',
  'crates/simulator-wasm',
  '--target',
  'web',
  '--release',
  '--out-dir',
  '../../src/wasm/pkg',
];

const result = spawnSync('wasm-pack', args, { stdio: 'inherit' });

if (result.error) {
  console.error('');
  console.error('Could not run wasm-pack. Install Rust and wasm-pack, then retry:');
  console.error('  rustup target add wasm32-unknown-unknown');
  console.error('  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh');
  console.error('');
  process.exit(1);
}

process.exit(result.status ?? 1);
