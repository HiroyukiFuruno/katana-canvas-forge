use crate::{
    DiagramKind, RenderDiagnostics, RenderError, RenderInput, RenderOutput, Renderer,
    RendererProfile, RuntimeVersion,
};
use hex;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct MermaidRenderer {
    pub vendor_dir: PathBuf,
    pub version: String,
}

impl MermaidRenderer {
    pub fn new(version: &str) -> Self {
        let vendor_dir = std::env::var("KCF_VENDOR_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."));

                if manifest_dir.ends_with("katana-canvas-forge") {
                    manifest_dir
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("vendor")
                } else {
                    manifest_dir.join("vendor")
                }
            });

        Self {
            vendor_dir,
            version: version.to_string(),
        }
    }
}

impl Renderer for MermaidRenderer {
    fn render(&self, input: &RenderInput) -> Result<RenderOutput, RenderError> {
        if input.kind != DiagramKind::Mermaid {
            return Err(RenderError::InvalidInput(
                "Expected Mermaid diagram".to_string(),
            ));
        }

        let render_js = self
            .vendor_dir
            .join("mermaid")
            .join(&self.version)
            .join("render.mjs");

        if !render_js.exists() {
            return Err(RenderError::Runtime(format!(
                "render.mjs not found at {:?}",
                render_js
            )));
        }

        let temp_file = NamedTempFile::new()
            .map_err(|e| RenderError::Runtime(format!("Failed to create temp file: {}", e)))?;
        let output_path = temp_file.path();

        let output = Command::new("node")
            .arg(render_js)
            .arg(&input.source)
            .arg(output_path)
            .output()
            .map_err(|e| RenderError::Runtime(format!("Failed to execute node: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(RenderError::Runtime(format!(
                "Node.js render failed.\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let svg = fs::read_to_string(output_path)
            .map_err(|e| RenderError::Runtime(format!("Failed to read output SVG: {}", e)))?;

        // Basic extraction of width, height, viewBox from SVG
        // In a real implementation, we might use an XML parser.
        let width = extract_attr(&svg, "width").unwrap_or(800.0);
        let height = extract_attr(&svg, "height").unwrap_or(600.0);
        let view_box =
            extract_attr_str(&svg, "viewBox").unwrap_or_else(|| "0 0 800 600".to_string());

        // Calculate stable cache fingerprint
        let mut hasher = Sha256::new();
        hasher.update(&self.version);
        hasher.update(&input.source);
        hasher.update(serde_json::to_string(&input.config).unwrap_or_default());
        let cache_fingerprint = hex::encode(hasher.finalize());

        Ok(RenderOutput {
            svg,
            width,
            height,
            view_box,
            runtime: RuntimeVersion {
                name: "mermaid-js".to_string(),
                version: self.version.clone(),
                checksum: Some("pinned-checksum".to_string()),
            },
            profile: RendererProfile {
                id: "mermaid-default".to_string(),
                description: Some("Default Mermaid.js renderer".to_string()),
            },
            diagnostics: RenderDiagnostics {
                warnings: vec![],
                errors: vec![],
            },
            cache_fingerprint,
        })
    }
}

fn extract_attr(svg: &str, attr: &str) -> Option<f32> {
    extract_attr_str(svg, attr)?.parse().ok()
}

fn extract_attr_str(svg: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = svg.find(&pattern)? + pattern.len();
    let end = svg[start..].find('"')?;
    Some(svg[start..start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderConfig, RenderContext, RenderPolicy};

    #[test]
    fn test_mermaid_renderer() {
        let renderer = MermaidRenderer::new("11.4.0");
        let input = RenderInput {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
            config: RenderConfig::default(),
            policy: RenderPolicy::default(),
            context: RenderContext::default(),
        };
        let output = renderer.render(&input).unwrap();
        assert!(output.svg.contains("rendered with mermaid.js"));
        assert!(!output.cache_fingerprint.is_empty());
    }
}
