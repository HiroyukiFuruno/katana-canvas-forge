//! katana-canvas-forge: versioned diagram rendering and document export runtime.
//!
//! This crate owns Mermaid / Draw.io rendering and HTML / PDF / PNG / JPEG export
//! responsibilities extracted from KatanA. KatanA consumes this crate as a
//! library through the renderer runtime interface defined here.
//!
//! Status: scaffolding. The runtime interface and Mermaid implementation are
//! migrated from KatanA in the v0.22.11 change.

pub mod exporter;
pub mod mermaid;
pub mod renderer;

pub const DEFAULT_MERMAID_VERSION: &str = "11.4.0";

pub use exporter::{ExportError, ExportFormat, ExportInput, ExportOutput, Exporter};
pub use renderer::{
    DiagramKind, RenderConfig, RenderContext, RenderDiagnostics, RenderError, RenderInput,
    RenderOutput, RenderPolicy, Renderer, RendererProfile, RuntimeVersion,
};
