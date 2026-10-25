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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match exhume::scan(&cli.path, &exhume::Options::default()) {
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
