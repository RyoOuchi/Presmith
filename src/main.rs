use clap::{Parser, Subcommand, ValueEnum};
use presmith::{self as app, process::INTERRUPTED};
use std::{path::PathBuf, process::ExitCode, sync::atomic::Ordering};
#[derive(Parser)]
#[command(
    version,
    about = "Source-first presentations for coding agents",
    long_about = "Create, preview, inspect and export local HTML presentations. Rust owns projects and serving; Node.js + Chromium power browser checks and rendering."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create a complete three-slide deck; never overwrites existing content
    Init { directory: PathBuf },
    /// Explicitly install pinned Node packages and project-local Chromium
    Setup {
        #[arg(default_value = ".")]
        directory: PathBuf,
        /// Back up and refresh bundled renderer files before installing dependencies
        #[arg(long)]
        upgrade_renderer: bool,
    },
    /// Check configuration and runtime capabilities without installing anything
    Doctor {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Serve on loopback with live rebuilds and slide-preserving reload
    Dev {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, default_value_t = 4173)]
        port: u16,
        #[arg(long)]
        open: bool,
    },
    /// Open the visual editor with explicit, conflict-checked source saves
    Edit {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, default_value_t = 4174)]
        port: u16,
        #[arg(long)]
        open: bool,
        /// Roll back an interrupted editor save before opening (close other editors first)
        #[arg(long)]
        recover: bool,
    },
    /// Capture logical-size slide PNGs and a labeled contact sheet
    Render {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        slide: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Validate sources and inspect each active slide in Chromium
    Check {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        slide: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Export a verified static HTML directory or one-page-per-slide PDF
    Export {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, value_enum)]
        format: Format,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Clone, ValueEnum)]
enum Format {
    Html,
    Pdf,
    Pptx,
}
fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            if e.use_stderr() && std::env::args().any(|a| a == "--json") {
                let (mut v, _) = app::failure("arguments", &anyhow::anyhow!(e.to_string()));
                v["error"]["kind"] = serde_json::json!("usage");
                println!("{v}");
                return ExitCode::from(2);
            }
            let code = e.exit_code();
            let _ = e.print();
            return ExitCode::from(code as u8);
        }
    };
    if let Err(e) = ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::Relaxed)) {
        eprintln!("Could not install signal handler: {e}");
        return ExitCode::from(2);
    }
    let (name, json, result) = match cli.command {
        Commands::Init { directory } => (
            "init",
            false,
            app::init(&directory).map(|()| app::envelope("init")),
        ),
        Commands::Setup {
            directory,
            upgrade_renderer,
        } => (
            "setup",
            false,
            app::setup_with_upgrade(&directory, upgrade_renderer).map(|()| app::envelope("setup")),
        ),
        Commands::Doctor { directory, json } => {
            let (v, code) = app::doctor(&directory);
            emit(&v, json);
            return ExitCode::from(code as u8);
        }
        Commands::Dev {
            directory,
            port,
            open,
        } => (
            "dev",
            false,
            app::dev(&directory, port, open).map(|()| app::envelope("dev")),
        ),
        Commands::Edit {
            directory,
            port,
            open,
            recover,
        } => (
            "edit",
            false,
            app::editor::edit(&directory, port, open, recover).map(|()| app::envelope("edit")),
        ),
        Commands::Render {
            directory,
            slide,
            out,
            json,
        } => (
            "render",
            json,
            app::browser_command("render", &directory, slide.as_deref(), out.as_deref(), None),
        ),
        Commands::Check {
            directory,
            slide,
            json,
        } => (
            "check",
            json,
            app::browser_command("check", &directory, slide.as_deref(), None, None),
        ),
        Commands::Export {
            directory,
            format,
            out,
            json,
        } => (
            "export",
            json,
            app::browser_command(
                "export",
                &directory,
                None,
                out.as_deref(),
                Some(match format {
                    Format::Html => "html",
                    Format::Pdf => "pdf",
                    Format::Pptx => "pptx",
                }),
            ),
        ),
    };
    let (v, code) = match result {
        Ok(v) => {
            let c = app::result_code(&v);
            (v, c)
        }
        Err(e) => app::failure(name, &e),
    };
    emit(&v, json);
    ExitCode::from(if INTERRUPTED.load(Ordering::Relaxed) {
        130
    } else {
        code as u8
    })
}
fn emit(v: &serde_json::Value, json: bool) {
    if json {
        println!("{v}");
        return;
    }
    if let Some(items) = v["findings"].as_array() {
        for f in items {
            eprintln!(
                "{} [{}] {} {}: {}",
                f["severity"].as_str().unwrap_or("error"),
                f["rule_id"].as_str().unwrap_or(""),
                f["slide_id"].as_str().unwrap_or("deck"),
                f["element_id"].as_str().unwrap_or(""),
                f["message"].as_str().unwrap_or("")
            );
        }
    }
    if let Some(items) = v["artifacts"].as_array() {
        for a in items {
            eprintln!(
                "{}: {}",
                a["kind"].as_str().unwrap_or("artifact"),
                a["path"].as_str().unwrap_or("")
            );
        }
    }
    if !v["capabilities"].is_null() {
        eprintln!("Capabilities: {}", v["capabilities"]);
    }
    if let Some(error) = v["error"]["message"].as_str() {
        eprintln!("{error}");
    }
    if v["success"] == true {
        eprintln!("{}: OK", v["command"].as_str().unwrap_or("presmith"));
    }
}
