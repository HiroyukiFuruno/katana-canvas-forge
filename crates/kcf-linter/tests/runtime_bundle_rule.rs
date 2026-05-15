use kcf_linter::{KcfLintError, KcfLinter, ViolationReport};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn reports_violation_for_include_str_from_js_runtime() -> TestResult<()> {
    let root = temp_root("runtime-include-jsruntime");
    write_valid_manifests(&root)?;
    // Split the macro name to avoid triggering the linter on this test file itself.
    let macro_call = ["include", "_str!(\"js_runtime/foo.js\");"].join("");
    let content = format!("pub const BAD: &str = {macro_call}");
    write_file(
        &root,
        "crates/katana-canvas-forge/src/bad_include.rs",
        &content,
    )?;

    let violations = KcfLinter::lint_workspace(&root)?;
    let report = ViolationReport::format(&violations);
    assert!(report.contains("runtime-bundle-boundary"), "{report}");
    assert!(
        report.contains("V8 runtime code must be included from generated bundles"),
        "{report}"
    );
    Ok(())
}

#[test]
fn reports_violation_for_include_str_from_diagram_runtime_source() -> TestResult<()> {
    let root = temp_root("runtime-include-source");
    write_valid_manifests(&root)?;
    // Split the macro name to avoid triggering the linter on this test file itself.
    let macro_call = ["include", "_str!(\"diagram_runtime/source/foo.ts\");"].join("");
    let content = format!("pub const BAD: &str = {macro_call}");
    write_file(
        &root,
        "crates/katana-canvas-forge/src/bad_include.rs",
        &content,
    )?;

    let violations = KcfLinter::lint_workspace(&root)?;
    let report = ViolationReport::format(&violations);
    assert!(report.contains("runtime-bundle-boundary"), "{report}");
    assert!(
        report.contains("V8 runtime code must be included from generated bundles"),
        "{report}"
    );
    Ok(())
}

#[test]
fn reports_walk_error_for_unreadable_runtime_source() -> TestResult<()> {
    // Use `scripts/` (TS_SCRIPT_ROOT) for the walk error, because
    // `crates/katana-canvas-forge/src` is already walked by WorkspaceModel::load
    // before RuntimeBundleRule runs. A blocked subdir inside `scripts/` is only
    // visited by RuntimeBundleRule::ts_files, triggering mod.rs:94-96.
    let root = temp_root("runtime-source-walk");
    write_valid_manifests(&root)?;
    let scripts = root.join("scripts/runtime-bundles");
    std::fs::create_dir_all(&scripts)?;
    let blocked = scripts.join("blocked");
    std::fs::create_dir_all(&blocked)?;
    set_mode(&blocked, 0o000)?;

    let result = KcfLinter::lint_workspace(&root);
    set_mode(&blocked, 0o755)?;
    assert!(matches!(result, Err(KcfLintError::Walk { .. })));
    Ok(())
}

#[test]
fn reports_read_error_for_unreadable_runtime_source_file() -> TestResult<()> {
    let root = temp_root("runtime-source-read");
    write_valid_manifests(&root)?;
    let file = write_file(
        &root,
        "crates/katana-canvas-forge/src/markdown/diagram_runtime/source/shared/forbidden.ts",
        "export const x = 1;",
    )?;
    set_mode(&file, 0o000)?;

    let result = KcfLinter::lint_workspace(&root);
    set_mode(&file, 0o644)?;
    assert!(matches!(result, Err(KcfLintError::Read { .. })));
    Ok(())
}

#[test]
fn reports_violation_for_denied_cross_runtime_import() -> TestResult<()> {
    let root = temp_root("runtime-cross-import");
    write_valid_manifests(&root)?;
    write_file(
        &root,
        "crates/katana-canvas-forge/src/markdown/diagram_runtime/source/mermaid/cross.ts",
        "import { foo } from \"../drawio/foo\";\nexport const BAR = foo;\n",
    )?;

    let violations = KcfLinter::lint_workspace(&root)?;
    let report = ViolationReport::format(&violations);
    assert!(
        report.contains("runtime source directories must depend only on shared helpers"),
        "{report}"
    );
    Ok(())
}

#[test]
fn reports_violation_for_forbidden_typescript_token() -> TestResult<()> {
    let root = temp_root("runtime-forbidden-token");
    write_valid_manifests(&root)?;
    write_file(
        &root,
        "crates/katana-canvas-forge/src/markdown/diagram_runtime/source/shared/forbidden_token.ts",
        "export function bad(input: any) {\n  return input;\n}\n",
    )?;

    let violations = KcfLinter::lint_workspace(&root)?;
    let report = ViolationReport::format(&violations);
    assert!(
        report.contains("TypeScript runtime gate forbids"),
        "{report}"
    );
    Ok(())
}

fn temp_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("kcf-linter-runtime-{name}-{}", std::process::id()))
}

fn write_valid_manifests(root: &Path) -> TestResult<()> {
    write_file(
        root,
        "crates/katana-canvas-forge/Cargo.toml",
        lib_manifest(),
    )?;
    write_file(
        root,
        "crates/katana-canvas-forge-cli/Cargo.toml",
        cli_manifest(),
    )?;
    Ok(())
}

fn lib_manifest() -> &'static str {
    r#"
[package]
name = "katana-canvas-forge"
version = "0.1.0"
edition = "2024"
"#
}

fn cli_manifest() -> &'static str {
    r#"
[package]
name = "katana-canvas-forge-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
katana-canvas-forge = { path = "../katana-canvas-forge" }
"#
}

fn write_file(root: &Path, relative: &str, content: &str) -> TestResult<PathBuf> {
    let path = root.join(relative);
    let Some(parent) = path.parent() else {
        return Err(Box::new(std::io::Error::other("path has no parent")));
    };
    std::fs::create_dir_all(parent)?;
    std::fs::write(&path, content)?;
    Ok(path)
}

fn set_mode(path: &Path, mode: u32) -> TestResult<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}
