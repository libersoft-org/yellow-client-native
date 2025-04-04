import { execSync } from 'child_process';
import path from 'path';

const clientPath = path.resolve(__dirname, '../yellow-client');

execSync('bun run build', {
    cwd: clientPath,
    env: { ...process.env, TAURI: 'true' },
    stdio: 'inherit'
});

// const targetPath = path.join(nativePath, 'build-tauri');
// console.log(`Copying build from ${buildPath} to ${targetPath}`);
// copySync(buildPath, targetPath);
//
