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
        #[clap(help = "Whether ")]
        // make a flag --check with a required string argument representing the git branch to check against
        #[clap(long, short, help = "Check if any service files have changed since the given git branch")]
        check: Option<String>,

        // create a catchall for any other args passed
        #[clap(last = true)]
        #[clap(help = "Any other args you want to pass to the docker build command (tags, etc)")]
        docker_build_args: Vec<String>,
    },
    #[clap(about = "Empties Bobpy's cache")]
    Clean,
}

pub fn parse() -> Args {
    Args::parse()
}

