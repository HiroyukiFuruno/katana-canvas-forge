use crate::{
    DiagramKind, RenderDiagnostics, RenderError, RenderInput, RenderOutput, Renderer,
    RendererProfile, RuntimeVersion,
};
use hex;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

pub struct MermaidRenderer {
    pub vendor_dir: PathBuf,
    pub version: String,
}

impl MermaidRenderer {
    pub fn new(version: &str) -> Self {
        // Find the vendor directory by checking common locations.
        let vendor_dir = std::env::var("KCF_VENDOR_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut search_paths = Vec::new();

                if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                    search_paths.push(PathBuf::from(manifest_dir));
                }
                if let Ok(cwd) = std::env::current_dir() {
                    search_paths.push(cwd);
                }

                for start_path in search_paths {
                    let mut current = Some(start_path.as_path());
                    while let Some(path) = current {
                        let maybe_vendor = path.join("vendor");
                        if maybe_vendor.exists() && maybe_vendor.is_dir() {
                            return maybe_vendor;
                        }
                        current = path.parent();
                    }
                }

                PathBuf::from("vendor") // Fallback
            });

        Self {
            vendor_dir,
            version: version.to_string(),
        }
    }

    fn get_checksum(&self) -> Option<String> {
        let sha256_path = self
            .vendor_dir
            .join("mermaid")
            .join(&self.version)
            .join("mermaid.min.js.sha256");

        if sha256_path.exists() {
            fs::read_to_string(sha256_path)
                .ok()
                .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        } else {
            // Fallback: compute it
            let js_path = self
                .vendor_dir
                .join("mermaid")
                .join(&self.version)
                .join("mermaid.min.js");
            fs::read(js_path).ok().map(|content| {
                let mut hasher = Sha256::new();
                hasher.update(content);
                hex::encode(hasher.finalize())
            })
        }
    }

    fn verify_node_version(&self) -> Result<(), RenderError> {
        let output = Command::new("node")
            .arg("--version")
            .output()
            .map_err(|e| {
                RenderError::Runtime(format!("Failed to execute node --version: {}", e))
            })?;

        if !output.status.success() {
            return Err(RenderError::Runtime("Node.js is not installed".to_string()));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        if !version_str.contains("v24") {
            // Warning for now, or could be error. Comment says "ensure node 24"
            tracing::warn!(
                "Node.js version {} detected. v24 is recommended.",
                version_str.trim()
            );
        }

        Ok(())
    }
}

impl Renderer for MermaidRenderer {
    fn render(&self, input: &RenderInput) -> Result<RenderOutput, RenderError> {
        if input.kind != DiagramKind::Mermaid {
            return Err(RenderError::InvalidInput(
                "Expected Mermaid diagram".to_string(),
            ));
        }

        self.verify_node_version()?;

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

        let temp_file = Builder::new()
            .suffix(".svg")
            .tempfile()
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
        let mut diagnostics = RenderDiagnostics {
            warnings: vec![],
            errors: vec![],
        };

        let width = extract_attr(&svg, "width").unwrap_or_else(|| {
            diagnostics
                .warnings
                .push("Width missing from SVG".to_string());
            800.0
        });
        let height = extract_attr(&svg, "height").unwrap_or_else(|| {
            diagnostics
                .warnings
                .push("Height missing from SVG".to_string());
            600.0
        });
        let view_box = extract_attr_str(&svg, "viewBox").unwrap_or_else(|| {
            diagnostics
                .warnings
                .push("viewBox missing from SVG".to_string());
            "0 0 800 600".to_string()
        });

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
                checksum: self.get_checksum(),
            },
            profile: RendererProfile {
                id: "mermaid-default".to_string(),
                description: Some("Default Mermaid.js renderer".to_string()),
            },
            diagnostics,
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
        assert!(output.svg.contains("svg"));
        assert!(!output.cache_fingerprint.is_empty());
        assert!(output.runtime.checksum.is_some());
    }
}
