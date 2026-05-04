# katana-canvas-forge justfile

MERMAID_VERSION := "11.4.0"

mermaid-js-update version=MERMAID_VERSION:
    @echo "Updating Mermaid.js to {{version}}..."
    mkdir -p vendor/mermaid/{{version}}
    # In a real scenario, we would download it:
    # curl -L https://unpkg.com/mermaid@{{version}}/dist/mermaid.min.js -o vendor/mermaid/{{version}}/mermaid.min.js
    # For v0.1.0 we have it placeholder/mocked.
    sha256sum vendor/mermaid/{{version}}/mermaid.min.js > vendor/mermaid/{{version}}/mermaid.min.js.sha256
    @echo "Updating references..."
    cargo run -p katana-canvas-forge-cli -- mermaid reference-update --fixtures tests/fixtures/mermaid_all --mermaid-version {{version}}
    @echo "Done."

compare:
    cargo run -p katana-canvas-forge-cli -- mermaid compare --fixtures tests/fixtures/mermaid_all --min-score 100

bench:
    cargo run -p katana-canvas-forge-cli -- mermaid bench --fixtures tests/fixtures/mermaid_all
