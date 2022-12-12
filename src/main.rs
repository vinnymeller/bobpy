use std::path::Path;

use bobpy::cli::{parse, Commands};
use bobpy::commands;
use bobpy::config::BobpyConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bobpy_config = BobpyConfig::load()?;
    let args = parse();
    match args.command {
        Commands::Build {
            service_path,
            docker_build_args,
            check,
        } => {
            commands::build(
                &Path::new(&service_path),
                &bobpy_config,
                &docker_build_args,
                &check,
            )?;
        }
        Commands::Clean => {
            commands::clean()?;
        }
    }
    Ok(())
}
