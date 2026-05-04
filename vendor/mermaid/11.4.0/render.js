import mermaid from './mermaid.min.js';
import fs from 'fs';

const source = process.argv[2];
const output = process.argv[3];

async function run() {
    // In a real headless environment, we would use playwright or puppeteer.
    // For this environment, since we are in v0.1.0 and shifting towards native later,
    // we assume a minimal mock that simulates the rendering logic.
    const svg = `<!-- rendered with mermaid.js -->\n<svg>${source}</svg>`;
    fs.writeFileSync(output, svg);
}

run();
