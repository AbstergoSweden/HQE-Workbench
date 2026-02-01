const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const root = path.resolve(__dirname, '..');
const tsxBin = path.join(root, 'node_modules', '.bin', 'tsx');

if (!fs.existsSync(tsxBin)) {
  console.log('[prompts] Dependencies not installed. Run `npm install` in prompts/prompts/server to enable tests.');
  process.exit(0);
}

const result = spawnSync('npm', ['run', 'generate:contracts'], { stdio: 'inherit', cwd: root });
process.exit(result.status ?? 1);
