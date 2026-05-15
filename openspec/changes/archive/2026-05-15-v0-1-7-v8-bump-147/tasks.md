# Tasks: katana-canvas-forge v0.1.7 v8 147 bump

- **標準ブランチ**: `release/v0.1.7`
- **作業ブランチ**: `feature/v0.1.7-task-x`
- **完了基準**: 全タスクが `[x]`、品質ゲート通過、PR マージ
- **関連 issue**: [#15](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/15)
- **参考実装**: kdr `824c340 fix: v8 147 APIに対応` + `4082dd9 chore(deps): bump v8`

---

## Phase 1: workspace 依存の bump

**目的**: workspace dep の v8 を 147 へ揃え、不要になった `mathjax_svg` を落とす。

- [x] 1.1 `Cargo.toml`（workspace）の `v8 = "=139.0.0"` を `"=147.4.0"` に変更する
- [x] 1.2 `Cargo.toml`（workspace）の `mathjax_svg = "3.2.0"` 行を削除する
- [x] 1.3 `crates/katana-canvas-forge/Cargo.toml` の `mathjax_svg = { workspace = true }` を削除する
- [x] 1.4 `cargo update -p v8` または `cargo build` で `Cargo.lock` を更新する

---

## Phase 2: V8 ランタイム実装の API 追従

**目的**: `diagram_js_runtime.rs` を v8 147 の新 API に対応させる。

- [x] 2.1 `V8_INIT` を `OnceLock<Result<(), String>>` から `OnceLock<()>` へ変更する
- [x] 2.2 `ensure_initialized()` を `V8::initialize_platform` + `V8::initialize` 直接呼びへ書き換える
- [x] 2.3 `mathjax_svg::convert_to_svg_inline` 呼出と `shared_runtime_init_result` ヘルパを削除する
- [x] 2.4 `DiagramTryCatchScope` 型エイリアスを追加する
- [x] 2.5 `HandleScope::new` / `TryCatch::new` を `v8::scope!` / `v8::tc_scope!` マクロへ置換する
- [x] 2.6 `evaluate` / `resolve_value` / `drain_microtasks` / `exception_message` の引数型を
       `&mut DiagramTryCatchScope<'_, '_, '_, '_>` へ置換する
- [x] 2.7 `drain_microtasks` 内の `perform_microtask_checkpoint` 呼出を `scope.as_mut().perform_microtask_checkpoint()` に変更する
- [x] 2.8 `script_origin` ヘルパを `evaluate` 内へインライン化する

---

## Phase 3: テストの追従

**目的**: `diagram_js_runtime_tests.rs` を新 API と削除済みヘルパに合わせる。

- [x] 3.1 `use super::{..., shared_runtime_init_result}` から該当 import を外す
- [x] 3.2 `allocation_and_init_error_helpers_are_error_first` を
       `allocation_helper_is_error_first` 相当に縮退させる（`shared_runtime_init_result` 検証を削除）
- [x] 3.3 スコープ生成（`HandleScope::new` 等）を新 API へ追従する
- [x] 3.4 `cargo test -p katana-canvas-forge` で V8 関連テストが通ることを確認する

---

## Phase 4: 上位レイヤーの回帰確認

**目的**: Mermaid / Draw.io / ZenUML / export が v8 147 上で動作することを確認する。

- [x] 4.1 `cargo build -p katana-canvas-forge` がエラーなく通ることを確認する
- [x] 4.2 `cargo test -p katana-canvas-forge` 全件通過を確認する
- [x] 4.3 `cargo test -p katana-canvas-forge-cli` 全件通過を確認する
- [x] 4.4 `just lint` を実行し warning ゼロを確認する
- [x] 4.5 `just mermaid-compare` / `just drawio-compare` が既存 baseline を維持することを確認する
       （baseline 改善は本 change のスコープ外）

---

## Phase 5: バージョン bump と仕上げ

- [x] 5.1 workspace `Cargo.toml` の `version` を `0.1.6` → `0.1.7` に上げる
- [x] 5.2 `just VERSION=v0.1.7 release-check` が通ることを確認する（archive 後）
- [x] 5.3 本 tasks.md の全タスクが `[x]` であることを確認する
