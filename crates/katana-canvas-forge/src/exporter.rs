use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Html,
    /// Planned for future release.
    Pdf,
    /// Planned for future release.
    Png,
    /// Planned for future release.
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInput {
    pub format: ExportFormat,
    pub svg: String,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOutput {
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("io error: {0}")]
    Io(String),
    #[error("runtime error: {0}")]
    Runtime(String),
}

pub trait Exporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>;
}

pub struct HtmlExporter;

impl Exporter for HtmlExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        match input.format {
            ExportFormat::Html => {
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Exported Diagram</title>
    <style>
        body {{ display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f5f5f5; }}
        .container {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
    </style>
</head>
<body>
    <div class="container">
        {}
    </div>
</body>
</html>"#,
                    input.svg
                );
                fs::write(&input.output_path, html).map_err(|e| ExportError::Io(e.to_string()))?;
                Ok(ExportOutput {
                    path: input.output_path.clone(),
                })
            }
            _ => Err(ExportError::UnsupportedFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_html_exporter() {
        let exporter = HtmlExporter;
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test.html");

        let input = ExportInput {
            format: ExportFormat::Html,
            svg: "<svg>test</svg>".to_string(),
            output_path: output_path.clone(),
        };
        let output = exporter.export(&input).unwrap();
        assert_eq!(output.path, output_path);
        assert!(output_path.exists());
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("<svg>test</svg>"));
    }

    #[test]
    fn test_unsupported_format() {
        let exporter = HtmlExporter;
        let input = ExportInput {
            format: ExportFormat::Pdf,
            svg: "<svg></svg>".to_string(),
            output_path: PathBuf::from("test.pdf"),
        };
        let result = exporter.export(&input);
        assert!(matches!(result, Err(ExportError::UnsupportedFormat)));
    }
}
