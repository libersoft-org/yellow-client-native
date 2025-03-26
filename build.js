import { execSync } from 'child_process';
import { copySync } from 'fs-extra';
import path from 'path';

const clientPath = path.resolve(__dirname, '../yellow-client');
const buildPath = path.join(clientPath, 'build');
const nativePath = path.resolve(__dirname, '../yellow-client-native');

execSync('bun run build', {
    cwd: clientPath,
    env: { ...process.env, TAURI: 'true' },
    stdio: 'inherit'
});

copySync(buildPath, nativePath + '/');
