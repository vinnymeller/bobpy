use bobpy::build::{build_service, clean_bobpy_cache};
use bobpy::cli::{parse, Commands};
use bobpy::config::get_settings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse();
    let settings = get_settings()?;
    match args.command {
        Commands::Build {
            service_path,
            docker_build_args,
        } => {
            build_service(service_path, docker_build_args, settings.requirement_lock)?;
        }
        Commands::Clean => {
            clean_bobpy_cache()?;
        }
        _ => {
            println!("Not implemented");
        }
    }

    Ok(())
}
