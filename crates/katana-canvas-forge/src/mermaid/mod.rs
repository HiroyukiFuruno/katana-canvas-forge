use crate::{
    DiagramKind, RenderDiagnostics, RenderError, RenderInput, RenderOutput, Renderer,
    RendererProfile, RuntimeVersion,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct MermaidRenderer {
    pub vendor_dir: PathBuf,
    pub version: String,
}

impl MermaidRenderer {
    pub fn new(version: &str) -> Self {
        // Default to a path that works in the repository structure.
        // In a real installed scenario, this might be configured via RenderConfig or env.
        let vendor_dir = std::env::var("KCF_VENDOR_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."));

                // If we are in crates/katana-canvas-forge, go up to workspace root
                if manifest_dir.ends_with("katana-canvas-forge") {
                    manifest_dir.parent().unwrap().parent().unwrap().join("vendor")
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

        let render_js = self.vendor_dir
            .join("mermaid")
            .join(&self.version)
            .join("render.js");

        if !render_js.exists() {
            return Err(RenderError::Runtime(format!(
                "render.js not found at {:?}",
                render_js
            )));
        }

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("mermaid_output_{}.svg", uuid::Uuid::new_v4()));

        let status = Command::new("node")
            .arg(render_js)
            .arg(&input.source)
            .arg(&output_path)
            .status()
            .map_err(|e| RenderError::Runtime(format!("Failed to execute node: {}", e)))?;

        if !status.success() {
            return Err(RenderError::Runtime("Node.js render failed".to_string()));
        }

        let svg = fs::read_to_string(&output_path)
            .map_err(|e| RenderError::Runtime(format!("Failed to read output SVG: {}", e)))?;

        let _ = fs::remove_file(output_path);

        Ok(RenderOutput {
            svg,
            width: 800.0,
            height: 600.0,
            view_box: "0 0 800 600".to_string(),
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
            cache_fingerprint: "stable-fingerprint".to_string(),
        })
    }
}

// Internal mock uuid-like thing for temp files if we don't want to add dependency
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> String {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string()
        }
    }
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
    }
}
