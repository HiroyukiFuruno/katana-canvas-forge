use clap::{Parser, Subcommand};
use katana_canvas_forge::mermaid::MermaidRenderer;
use katana_canvas_forge::{
    DEFAULT_MERMAID_VERSION, DiagramKind, RenderConfig, RenderContext, RenderInput, RenderPolicy,
    Renderer,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

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
        #[arg(long, default_value = DEFAULT_MERMAID_VERSION)]
        mermaid_version: String,
    },
    ReferenceUpdate {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value = DEFAULT_MERMAID_VERSION)]
        mermaid_version: String,
    },
    Compare {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value_t = 99.0)]
        min_score: f32,
        #[arg(long, default_value = DEFAULT_MERMAID_VERSION)]
        mermaid_version: String,
    },
    Bench {
        #[arg(long)]
        fixtures: String,
        #[arg(long, default_value = DEFAULT_MERMAID_VERSION)]
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
                println!("Rendered {} to {}", input, output);
            }
            MermaidAction::ReferenceUpdate {
                fixtures,
                mermaid_version,
            } => {
                let fixtures_path = Path::new(&fixtures);
                let renderer = MermaidRenderer::new(&mermaid_version);

                for entry in fs::read_dir(fixtures_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "mmd") {
                        let source = fs::read_to_string(&path)?;
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
                        let mut ref_path = path.clone();
                        ref_path.set_extension("svg");
                        fs::write(&ref_path, render_output.svg)?;
                        println!("Updated reference for {:?}", path.file_name().unwrap());
                    }
                }
            }
            MermaidAction::Compare {
                fixtures,
                min_score,
                mermaid_version,
            } => {
                let fixtures_path = Path::new(&fixtures);
                let renderer = MermaidRenderer::new(&mermaid_version);
                let mut total_count = 0;
                let mut fail_count = 0;

                for entry in fs::read_dir(fixtures_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "mmd") {
                        total_count += 1;
                        let source = fs::read_to_string(&path)?;
                        let mut ref_path = path.clone();
                        ref_path.set_extension("svg");

                        if !ref_path.exists() {
                            println!(
                                "FAIL: Reference not found for {:?}",
                                path.file_name().unwrap()
                            );
                            fail_count += 1;
                            continue;
                        }

                        let reference_svg = fs::read_to_string(&ref_path)?;
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

                        let score = calculate_score(&render_output.svg, &reference_svg);
                        if score < min_score {
                            println!(
                                "FAIL: {:?} score {:.2} (min {:.2})",
                                path.file_name().unwrap(),
                                score,
                                min_score
                            );
                            fail_count += 1;
                        } else {
                            println!("PASS: {:?} score {:.2}", path.file_name().unwrap(), score);
                        }
                    }
                }
                println!(
                    "Result: {}/{} passed",
                    total_count - fail_count,
                    total_count
                );
                if fail_count > 0 {
                    anyhow::bail!("Comparison failed for {} fixtures", fail_count);
                }
            }
            MermaidAction::Bench {
                fixtures,
                mermaid_version,
            } => {
                let fixtures_path = Path::new(&fixtures);
                let renderer = MermaidRenderer::new(&mermaid_version);

                for entry in fs::read_dir(fixtures_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "mmd") {
                        let source = fs::read_to_string(&path)?;
                        let render_input = RenderInput {
                            kind: DiagramKind::Mermaid,
                            source,
                            config: RenderConfig::default(),
                            policy: RenderPolicy::default(),
                            context: RenderContext::default(),
                        };

                        let start = Instant::now();
                        let _ = renderer
                            .render(&render_input)
                            .map_err(|e| anyhow::anyhow!(e))?;
                        let duration = start.elapsed();
                        println!("Bench: {:?} took {:?}", path.file_name().unwrap(), duration);
                    }
                }
            }
        },
    }
    Ok(())
}

fn calculate_score(actual: &str, expected: &str) -> f32 {
    if actual == expected {
        return 100.0;
    }
    // Very primitive scoring for v0.1.0:
    // If it's not exact, we give it a 0.0 or we could do something more.
    // Given the min_score 99 requirement, they probably expect high fidelity.
    0.0
}
