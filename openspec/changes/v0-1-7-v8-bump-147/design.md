## 設計

### 問題の構造

v0.1.6 までの `DiagramV8Runtime::ensure_initialized` は、`mathjax_svg::convert_to_svg_inline("x")`
を OnceLock 経由で 1 回だけ呼ぶことで V8 platform を間接的に初期化していた。
これは `rusty_v8` が global state を持ち、二重初期化で panic する制約への
workaround であり、`mathjax_svg` は副作用としてだけ使われていた。

v8 147 では以下の API 変更が入った（kdr `824c340` で確認）:

- 明示的な platform 初期化 API: `v8::new_default_platform(0, false).make_shared()`
  → `v8::V8::initialize_platform(platform)` → `v8::V8::initialize()`
- `HandleScope` / `TryCatch` 等の lifetime を扱う `PinnedRef` 型と、
  対応するマクロ `v8::scope!` / `v8::tc_scope!` が導入された
- `TryCatch<HandleScope>` への `&mut` 操作は `scope.as_mut()` 経由で行う
- `ScriptOrigin::new` のシグネチャ自体は同一

これらに合わせて、KCF 側の薄い抽象 `DiagramV8Runtime` を更新する。
上位レイヤー（mermaid / drawio / zenuml の各 renderer）は
`DiagramV8Runtime::render(&[DiagramRuntimeScript])` のみを利用しているため、
理屈の上では変更不要を見込む。

### 修正方針

1. **workspace deps**
   - `v8 = "=139.0.0"` → `v8 = "=147.4.0"`
   - `mathjax_svg = "3.2.0"` を削除
2. **`crates/katana-canvas-forge/Cargo.toml`**
   - `mathjax_svg = { workspace = true }` を削除
3. **`diagram_js_runtime.rs`**
   - `V8_INIT: OnceLock<()>` に簡略化（`Result<(), String>` ではなく）
   - `ensure_initialized()` を直接 `V8::initialize_platform` + `V8::initialize` で実装
   - `HandleScope::new` / `TryCatch::new` を `v8::scope!` / `v8::tc_scope!` マクロへ
   - 関数シグネチャの `&mut v8::TryCatch<v8::HandleScope>` を新しい
     `DiagramTryCatchScope` 型エイリアスに置換
   - `perform_microtask_checkpoint` 呼出に `.as_mut()` を挟む
   - `script_origin` ヘルパは `evaluate` 内へインライン化
   - `shared_runtime_init_result` ヘルパは削除
4. **`diagram_js_runtime_tests.rs`**
   - `shared_runtime_init_result` 関連テストを削除
   - スコープ生成を新 API へ追従

### インターフェース（抜粋）

```rust
// diagram_js_runtime.rs（変更後の骨子）
static V8_INIT: OnceLock<()> = OnceLock::new();

type DiagramTryCatchScope<'pin, 'scope, 'object, 'isolate> =
    v8::PinnedRef<'pin, v8::TryCatch<'scope, 'object, v8::HandleScope<'isolate>>>;

impl DiagramV8Runtime {
    pub(crate) fn render(scripts: &[DiagramRuntimeScript<'_>]) -> Result<String, String> {
        Self::ensure_initialized();

        let mut isolate = v8::Isolate::new(Default::default());
        v8::scope!(let handle_scope, &mut isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let context_scope = &mut v8::ContextScope::new(handle_scope, context);
        v8::tc_scope!(let scope, &mut **context_scope);
        // ...
    }

    fn ensure_initialized() {
        V8_INIT.get_or_init(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });
    }
}
```

上位レイヤー（mermaid_renderer / drawio_renderer / zenuml_v8_runtime）は
`DiagramV8Runtime::render` の呼び出しシグネチャを維持するため変更を要しない。
ただし `cargo build` でエラーが出る箇所があれば追加で追従する。

### 検証

- `cargo build -p katana-canvas-forge` がエラーなく通る
- `cargo test -p katana-canvas-forge` で V8 関連テストが全て通る
- `just lint` がエラーなく通る
- KatanA 側（参考）から KCF git dep で取り込んだ際に Mermaid render worker が
  panic しないことを手動確認（本 change の検証対象外、release 後に KatanA 側で確認）

### スコープ外

- Mermaid / Draw.io / ZenUML JS bundle 内容の変更
- HTML / PDF / PNG / JPEG export 経路の改修
- reference baseline の更新
- KatanA 側の Cargo.toml 更新（KCF 0.1.7 タグ作成後に KatanA 側で追従）

### リスク

- v8 147 で `Isolate::new` / `Context::new` の lifetime 周りが厳格化されている
  ため、`handle_scope` を `context_scope` へ渡す箇所でコンパイルエラーが出る
  可能性がある。kdr の追従パターンを踏襲することで回避する。
- `mathjax_svg` 削除に伴い「KCF 内で v8 を使う他箇所」が暗黙の初期化に依存
  していた場合に panic する可能性がある。`ensure_initialized` を呼び出す経路
  （`DiagramV8Runtime::render` 経由）に集約されていることを確認する。
