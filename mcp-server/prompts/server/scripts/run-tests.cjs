const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const tsxBin = path.join(root, 'node_modules', '.bin', 'tsx');

if (!fs.existsSync(tsxBin)) {
  console.log('[prompts] Dependencies not installed. Run `npm install` in prompts/prompts/server to enable tests.');
  process.exit(0);
}

const run = (cmd, args) => {
  const result = spawnSync(cmd, args, { stdio: 'inherit', cwd: root });
  if (result.status !== 0) {
    process.exit(result.status);
  }
};

run('npm', ['run', 'test:unit']);
