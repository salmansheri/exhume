use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Find unused files, exports, dependencies and env vars in a JS/TS workspace.
#[derive(Parser)]
#[command(name = "exhume", version)]
struct Cli {
    /// Root of the workspace to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Report unused Public API of published packages too.
    #[arg(long)]
    include_public_api: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = exhume::Options {
        include_public_api: cli.include_public_api,
    };
    match exhume::scan(&cli.path, &options) {
        Ok(report) => {
            print!("{}", report.to_text());
            if report.summary.total == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("exhume: {error:#}");
            ExitCode::from(2)
        }
    }
}
