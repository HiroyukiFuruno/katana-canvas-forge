# Spec: Diagram V8 runtime on v8 147

## 目的

KCF と `katana-diagram-renderer`（kdr）を同一 KatanA workspace に取り込んだとき、
`v8` crate が 1 バージョンに収束し、Mermaid / Draw.io / ZenUML の V8 ランタイムが
panic せずに描画を返すこと。

## 変更対象

| ファイル | 変更内容 |
|---------|---------|
| `Cargo.toml`（workspace） | `v8 = "=147.4.0"`、`mathjax_svg` 削除、`version = "0.1.7"` |
| `crates/katana-canvas-forge/Cargo.toml` | `mathjax_svg = { workspace = true }` 削除 |
| `crates/katana-canvas-forge/src/markdown/diagram_js_runtime.rs` | v8 147 API（`scope!` / `tc_scope!`、明示初期化）へ追従 |
| `crates/katana-canvas-forge/src/markdown/diagram_js_runtime_tests.rs` | 新 API への追従、`shared_runtime_init_result` テスト削除 |

## 契約

- `DiagramV8Runtime::render(scripts)` の公開シグネチャは変更しない。
  上位 renderer（mermaid / drawio / zenuml）は本 change で変更を受けない。
- V8 platform 初期化は `DiagramV8Runtime::ensure_initialized()` を経由する 1 経路に集約する。
  `mathjax_svg::convert_to_svg_inline` 経由の暗黙初期化は廃止する。
- `Cargo.toml` の workspace dep に `mathjax_svg` を含めない。
  もし将来 KCF 内で MathJax を SVG 化する必要が生じたら、その時点で別 change として再導入する。
- `v8` workspace dep は KCF と kdr の両方で `"=147.4.0"` を採用する。
  KCF 0.1.7 の release は kdr 0.1.0 と同じ v8 バージョンに揃えることを前提とする。

## 完了条件

- `cargo build -p katana-canvas-forge` がエラーなく通る。
- `cargo test -p katana-canvas-forge` の V8 関連テスト（`diagram_js_runtime_tests`,
  `mermaid_renderer::js_runtime_tests`, `zenuml_v8_runtime_tests`）が全て通る。
- `cargo test -p katana-canvas-forge-cli` が通る。
- `just lint` が warning ゼロで通る。
- workspace `Cargo.toml` の `version` が `0.1.7` になっている。
- `just mermaid-compare` / `just drawio-compare` が既存 baseline を下回らない。
