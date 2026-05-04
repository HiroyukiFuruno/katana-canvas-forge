import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const source = process.argv[2];
const output = process.argv[3];

async function run() {
    const tempMmd = output + ".mmd";
    fs.writeFileSync(tempMmd, source);

    try {
        // Search for node_modules upward
        let current = __dirname;
        let mmdcPath = null;
        while (current !== path.parse(current).root) {
            const potential = path.join(current, 'node_modules/.bin/mmdc');
            if (fs.existsSync(potential)) {
                mmdcPath = potential;
                break;
            }
            current = path.dirname(current);
        }

        if (!mmdcPath) {
             const absoluteAppMmdc = '/app/node_modules/.bin/mmdc';
             if (fs.existsSync(absoluteAppMmdc)) {
                 mmdcPath = absoluteAppMmdc;
             }
        }

        if (!mmdcPath) {
            throw new Error("mmdc not found in node_modules");
        }

        // Avoid -p /dev/null if it causes JSON parse error in some environments
        execSync(`${mmdcPath} -i ${tempMmd} -o ${output}`, { stdio: 'inherit' });
    } catch (err) {
        console.error("Mermaid render error:", err);
        process.exit(1);
    } finally {
        if (fs.existsSync(tempMmd)) {
            fs.unlinkSync(tempMmd);
        }
    }
}

run();
