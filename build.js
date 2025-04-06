import { execSync } from 'child_process';
import path from 'path';

//const clientPath = path.resolve(__dirname, '../yellow-client');
const clientPath = path.resolve('yellow-client');

execSync('bun install && bun run build', {
    cwd: clientPath,
    env: { ...process.env, TAURI: 'true' },
    stdio: 'inherit',
    shell: true,
});

// const targetPath = path.join(nativePath, 'build-tauri');
// console.log(`Copying build from ${buildPath} to ${targetPath}`);
// copySync(buildPath, targetPath);
//
