use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Html,
    Pdf,
    Png,
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExporter;

    impl Exporter for MockExporter {
        fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
            Ok(ExportOutput {
                path: input.output_path.clone(),
            })
        }
    }

    #[test]
    fn test_mock_exporter() {
        let exporter = MockExporter;
        let input = ExportInput {
            format: ExportFormat::Pdf,
            svg: "<svg></svg>".to_string(),
            output_path: PathBuf::from("test.pdf"),
        };
        let output = exporter.export(&input).unwrap();
        assert_eq!(output.path, PathBuf::from("test.pdf"));
    }
}
