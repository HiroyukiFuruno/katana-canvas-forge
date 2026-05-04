# katana-canvas-forge justfile

MERMAID_VERSION := "11.4.0"
FIXTURES_DIR := "tests/fixtures/mermaid_all"

# Default action: list recipes
default:
    @just --list

# Lint the codebase
lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
test:
    cargo test --workspace

# Update Mermaid.js bundle and reference images
mermaid-js-update version=MERMAID_VERSION:
    @echo "Updating Mermaid.js to {{version}}..."
    mkdir -p vendor/mermaid/{{version}}
    curl -L https://unpkg.com/mermaid@{{version}}/dist/mermaid.min.js -o vendor/mermaid/{{version}}/mermaid.min.js
    sha256sum vendor/mermaid/{{version}}/mermaid.min.js > vendor/mermaid/{{version}}/mermaid.min.js.sha256
    @echo "Updating references..."
    cargo run -p katana-canvas-forge-cli -- mermaid reference-update --fixtures {{FIXTURES_DIR}} --mermaid-version {{version}}
    @echo "Done."

# Render a single mermaid file
render input output version=MERMAID_VERSION:
    cargo run -p katana-canvas-forge-cli -- mermaid render --input {{input}} --output {{output}} --mermaid-version {{version}}

# Compare fixtures against reference images
compare min_score="100" version=MERMAID_VERSION:
    cargo run -p katana-canvas-forge-cli -- mermaid compare --fixtures {{FIXTURES_DIR}} --min-score {{min_score}} --mermaid-version {{version}}

# Benchmark rendering performance
bench version=MERMAID_VERSION:
    cargo run -p katana-canvas-forge-cli -- mermaid bench --fixtures {{FIXTURES_DIR}} --mermaid-version {{version}}

# Clean build artifacts
clean:
    cargo clean
