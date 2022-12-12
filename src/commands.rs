use crate::git;
use crate::{config::BobpyConfig, parsing};
use fs_extra::dir;
use std::path::Path;

pub fn build(
    service_path: &Path,
    bobpy_config: &BobpyConfig,
    docker_build_args: &Vec<String>,
    check: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_context = parsing::ServiceContext::from_path(service_path)?;
    if let Some(check) = check {
        let git_diff_paths = git::get_changed_git_paths(check)?;
        let service_paths = service_context.get_all_paths();
        if !git::any_service_files_changed(&git_diff_paths, &service_paths) {
            println!("No service files changed, skipping build");
            return Ok(());
        }
    }
    let build_path = service_context.write_service_context(bobpy_config)?;
    let mut docker_build = std::process::Command::new("docker")
        .arg("build")
        .arg(&build_path)
        .arg("--file")
        .arg(&build_path.join(service_path).join("Dockerfile"))
        .args(docker_build_args)
        .spawn()?;
    docker_build.wait()?;
    Ok(())
}

pub fn clean() -> Result<(), fs_extra::error::Error> {
    Ok(dir::remove(".bobpy")?)
}

