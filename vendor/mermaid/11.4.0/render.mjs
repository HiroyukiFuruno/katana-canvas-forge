import { JSDOM } from 'jsdom';
import fs from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const mermaidJsPath = path.join(__dirname, 'mermaid.min.js');

const source = process.argv[2];
const output = process.argv[3];

async function run() {
    const dom = new JSDOM('<!DOCTYPE html><html><body><div id="graph"></div></body></html>', {
        runScripts: "dangerously",
        resources: "usable"
    });

    // Shim getBBox for Mermaid in JSDOM
    // This is a minimal shim to prevent "getBBox is not a function" errors.
    // Real measurement is impossible in JSDOM, but Mermaid can often proceed with placeholders.
    if (!dom.window.SVGElement.prototype.getBBox) {
        dom.window.SVGElement.prototype.getBBox = function() {
            return {
                x: 0,
                y: 0,
                width: 100,
                height: 100
            };
        };
    }

    // We need to inject mermaid into the JSDOM window
    const scriptContent = fs.readFileSync(mermaidJsPath, 'utf8');
    const script = dom.window.document.createElement("script");
    script.textContent = scriptContent;
    dom.window.document.body.appendChild(script);

    const mermaid = dom.window.mermaid;

    mermaid.initialize({
        startOnLoad: false,
        domPurify: true,
        securityLevel: 'loose',
    });

    try {
        const { svg } = await mermaid.render('id1', source);
        fs.writeFileSync(output, svg);
    } catch (err) {
        console.error("Mermaid render error:", err);
        process.exit(1);
    }
}

run();
