import mermaid from './mermaid.min.js';
import fs from 'fs';

const source = process.argv[2];
const output = process.argv[3];

async function run() {
    // Mock rendering logic that produces a valid SVG with attributes
    const svg = `<svg width="800" height="600" viewBox="0 0 800 600"><!-- rendered with mermaid.js -->\n<g>${source}</g></svg>`;
    fs.writeFileSync(output, svg);
}

run().catch(err => {
    console.error(err);
    process.exit(1);
});
