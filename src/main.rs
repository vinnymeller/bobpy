use std::path::Path;

use bobpy::cli::{parse, Commands};
use bobpy::parsing::Service;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse();
    match args.command {
        Commands::Build {
            service_path,
            docker_build_args: _,
        } => {
            Service::from_path(Path::new(&service_path))?
                .create_service_context()?;
        }
        _ => {
            println!("Not implemented");
        }

    }
    Ok(())
}
