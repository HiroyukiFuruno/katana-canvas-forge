use crate::{
    DiagramKind, RenderDiagnostics, RenderError, RenderInput, RenderOutput, Renderer,
    RendererProfile, RuntimeVersion,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct MermaidRenderer {
    pub mermaid_js_path: PathBuf,
    pub version: String,
}

impl MermaidRenderer {
    pub fn new(version: &str) -> Self {
        let mermaid_js_path = PathBuf::from("vendor/mermaid")
            .join(version)
            .join("mermaid.min.js");
        Self {
            mermaid_js_path,
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

        // Search for vendor directory starting from current dir up to root
        let mut vendor_dir = PathBuf::from("vendor");
        if !vendor_dir.exists() {
            // Try relative to workspace root if we are in a crate subdir
            vendor_dir = PathBuf::from("../../vendor");
        }

        let render_js = vendor_dir
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
        let output_path = temp_dir.join("mermaid_output.svg");

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

        Ok(RenderOutput {
            svg,
            width: 800.0,
            height: 600.0,
            view_box: "0 0 800 600".to_string(),
            runtime: RuntimeVersion {
                name: "mermaid-js".to_string(),
                version: self.version.clone(),
                checksum: Some("mock-checksum".to_string()),
            },
            profile: RendererProfile {
                id: "mermaid-default".to_string(),
                description: Some("Default Mermaid.js renderer".to_string()),
            },
            diagnostics: RenderDiagnostics {
                warnings: vec![],
                errors: vec![],
            },
            cache_fingerprint: "mock-fingerprint".to_string(),
        })
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
