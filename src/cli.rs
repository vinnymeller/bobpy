use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Commands {
    #[clap(about = "Build a service")]
    Build {
        #[clap(help = "Path to the service you want to build")]
        service_path: String,
        // create a catchall for any other args passed
        #[clap(last = true)]
        #[clap(help = "Any other args you want to pass to the docker build command (tags, etc)")]
        docker_build_args: Vec<String>,
    },
    #[clap(about = "Check that all configuration for a service is valid")]
    Valid {
        #[clap(help = "Path to the service you want to check")]
        service_path: String,
    },
    #[clap(about = "Check if a service needs to be rebuilt")]
    Check {
        #[clap(help = "Path to the service you want to check")]
        service_path: String,
    },
    #[clap(about = "Empties Bobpy's cache")]
    Clean,
    #[clap(about = "Update the Bobpy binary")]
    Update,
}

pub fn parse() -> Args {
    Args::parse()
}
