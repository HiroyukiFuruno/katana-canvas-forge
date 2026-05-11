use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_js_runtime::{DiagramRuntimeScript, DiagramV8Runtime};
use crate::markdown::runtime_assets::RuntimeAsset;

const BROWSER_GLOBALS: &str = include_str!("js_runtime/browser_globals.js");
const BRIDGE_SCRIPT: &str = include_str!("js_runtime/render_zenuml_v8.js");

pub(super) struct ZenumlV8RenderOps;

impl ZenumlV8RenderOps {
    pub(super) fn render(
        source: &str,
        _preset: &DiagramColorPreset,
        _svg_id: String,
    ) -> Result<String, String> {
        let zenuml_asset = RuntimeAsset::zenuml_core();
        let zenuml_bundle = materialize_and_read(&zenuml_asset)?;
        let preamble = build_preamble(source)?;
        let scripts = [
            DiagramRuntimeScript::borrowed("browser-globals.js", BROWSER_GLOBALS),
            DiagramRuntimeScript::owned("zenuml-preamble.js", preamble),
            DiagramRuntimeScript::owned("zenuml.js", zenuml_bundle),
            DiagramRuntimeScript::borrowed("render-zenuml-v8.js", BRIDGE_SCRIPT),
        ];
        DiagramV8Runtime::render(&scripts)
    }
}

fn materialize_and_read(asset: &RuntimeAsset) -> Result<String, String> {
    let path = asset.materialize_at(asset.materialized_path())?;
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read zenuml.js: {e}"))
}

fn build_preamble(source: &str) -> Result<String, String> {
    let source_json = serde_json::to_string(source)
        .map_err(|e| format!("Failed to serialize zenuml source: {e}"))?;
    Ok(format!("var __zenuml_source__ = {source_json};"))
}

#[cfg(test)]
#[path = "zenuml_v8_runtime_tests.rs"]
mod tests;
