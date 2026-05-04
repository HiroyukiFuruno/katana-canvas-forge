use clap::{Parser, Subcommand};
use katana_canvas_forge::mermaid::MermaidRenderer;
use katana_canvas_forge::{
    DiagramKind, RenderConfig, RenderContext, RenderInput, RenderPolicy, Renderer,
};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "kcf", version, about = "katana-canvas-forge CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mermaid {
        #[command(subcommand)]
        action: MermaidAction,
    },
}

#[derive(Subcommand)]
enum MermaidAction {
    Render {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
        #[arg(long, default_value = "11.4.0")]
        mermaid_version: String,
    },
    ReferenceUpdate {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value = "11.4.0")]
        mermaid_version: String,
    },
    Compare {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value_t = 99.0)]
        min_score: f32,
        #[arg(long, default_value = "11.4.0")]
        mermaid_version: String,
    },
    Bench {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value = "11.4.0")]
        mermaid_version: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Mermaid { action } => match action {
            MermaidAction::Render {
                input,
                output,
                mermaid_version,
            } => {
                let source = fs::read_to_string(&input)?;
                let renderer = MermaidRenderer::new(&mermaid_version);
                let render_input = RenderInput {
                    kind: DiagramKind::Mermaid,
                    source,
                    config: RenderConfig::default(),
                    policy: RenderPolicy::default(),
                    context: RenderContext::default(),
                };
                let render_output = renderer
                    .render(&render_input)
                    .map_err(|e| anyhow::anyhow!(e))?;
                fs::write(&output, render_output.svg)?;
                println!("Rendered {input} to {output}");
            }
            MermaidAction::ReferenceUpdate {
                fixtures,
                mermaid_version,
            } => {
                let fixtures_path = Path::new(&fixtures);
                if !fixtures_path.exists() {
                    anyhow::bail!("Fixtures directory not found: {fixtures}");
                }
                println!("Updating references in {fixtures} using Mermaid {mermaid_version}...");
                // Placeholder for actual logic
            }
            MermaidAction::Compare {
                fixtures,
                min_score,
                mermaid_version,
            } => {
                println!(
                    "Comparing fixtures in {fixtures} (min-score={min_score}) using Mermaid {mermaid_version}..."
                );
                // Placeholder for actual logic
            }
            MermaidAction::Bench {
                fixtures,
                mermaid_version,
            } => {
                println!("Benchmarking fixtures in {fixtures} using Mermaid {mermaid_version}...");
                // Placeholder for actual logic
            }
        },
    }
    Ok(())
}
