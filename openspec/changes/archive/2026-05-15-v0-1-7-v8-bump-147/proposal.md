## Why

`katana-diagram-renderer`（kdr）0.1.0 が `v8 = "=147.4.0"` で publish された。
一方 KCF 0.1.6 は `v8 = "=139.0.0"` のため、両 crate を同一 workspace
（KatanA）に取り込むと `v8` が 2 バージョン共存し、V8 ランタイムワーカーが
起動時に panic する。

### 実害（issue #15）

- KatanA 上で Mermaid 図形がすべて以下のメッセージで失敗する:
  ```
  [Mermaid] Diagram render worker disconnected before producing a result.
  ```
- Draw.io / ZenUML も同じ V8 isolate を共有するため同様に失敗する想定。

### 関連リポジトリ状況

- kdr 側は HEAD（v0.1.0 publish 想定）で v8 = "=147.4.0" + API 追従済み
  （commit `824c340 fix: v8 147 APIに対応`、`4082dd9 chore(deps): bump v8`）。
- KCF 側で同等の v8 bump と API 追従を行えば、KatanA workspace で
  v8 を一本化できる。

## What Changes

| 領域 | 変更 |
|------|------|
| workspace `Cargo.toml` | `v8 = "=139.0.0"` → `"=147.4.0"`、`mathjax_svg = "3.2.0"` を削除 |
| `crates/katana-canvas-forge/Cargo.toml` | `mathjax_svg = { workspace = true }` を削除 |
| `diagram_js_runtime.rs` | v8 147 API（`v8::scope!` / `v8::tc_scope!`、`V8::initialize_platform` 明示初期化）へ追従 |
| `diagram_js_runtime_tests.rs` | 新 API への追従、`shared_runtime_init_result` テスト削除 |
| workspace `version` | `0.1.6` → `0.1.7` |

`mathjax_svg` は KCF 内では V8 共有初期化の副作用としてだけ参照されていた
（実際の数式描画は HTML テンプレートの KaTeX が担当）。v8 147 では
`v8::V8::initialize_platform` が明示的に呼べるため、mathjax_svg への依存は
不要となる。

## Deliverables

- `Cargo.toml` — workspace dep の v8 bump と mathjax_svg 削除、version bump
- `crates/katana-canvas-forge/Cargo.toml` — mathjax_svg 削除
- `crates/katana-canvas-forge/src/markdown/diagram_js_runtime.rs` — v8 147 API 追従
- `crates/katana-canvas-forge/src/markdown/diagram_js_runtime_tests.rs` — 同上
- `openspec/changes/v0-1-7-v8-bump-147/` — 本変更記録

## Out of Scope

- Mermaid / Draw.io / ZenUML の JS 実装側の変更（`DiagramV8Runtime` 抽象を
  経由するため、上位レイヤーは変更不要を期待）。
- HTML / PDF / PNG / JPEG export 機能の改修（回帰確認のみ）。
- reference score の baseline 更新（v8 bump 由来の差分が出た場合のみ別 change で対応）。
