use crate::config::RequirementLockMap;
use crate::parsing::{get_build_context, BuildContext};
use fs_extra::dir;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn create_bobpy_build_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bobpy_build_path = PathBuf::from(".")
        .join(".bobpy")
        .join("builds")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&bobpy_build_path)?;
    Ok(bobpy_build_path)
}

fn create_dirs_and_copy(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Copying {} to {}",
        from.to_str().unwrap(),
        to.to_str().unwrap()
    );
    // if we are copying a file, create the directory it is in
    if from.is_file() {
        fs::create_dir_all(to.parent().unwrap())?;
        fs::copy(from, to)?;
    } else {
        dir::create_all(to, false)?;
        dir::copy(
            from,
            to.parent().unwrap(),
            &dir::CopyOptions {
                copy_inside: true,
                skip_exist: true,
                ..dir::CopyOptions::new()
            },
        )?;
    }
    Ok(())
}

fn copy_files_to_build_dir(
    build_context: &BuildContext,
    build_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    create_dirs_and_copy(
        &build_context.build_path,
        build_path.join(&build_context.build_path).as_ref(),
    )?;
    for lib in build_context.libraries.iter() {
        create_dirs_and_copy(lib, build_path.join(lib).as_ref())?;
    }
    for path in build_context.paths.iter() {
        create_dirs_and_copy(path, build_path.join(path).as_ref())?;
    }
    Ok(())
}

fn run_docker_build(
    build_path: &Path,
    docker_build_args: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let build_path_str = build_path.to_str().unwrap();
    let mut docker_build = std::process::Command::new("docker")
        .arg("build")
        .arg(build_path_str)
        .arg("--file")
        .arg(build_path.join("Dockerfile"))
        .args(docker_build_args)
        .spawn()?;
    docker_build.wait()?;
    Ok(())
}

pub fn build_service(
    service_path: String,
    docker_build_args: Vec<String>,
    lock_map: RequirementLockMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let build_context = get_build_context(&service_path)?;
    let build_path = create_bobpy_build_dir()?;
    let service_path = &build_path.join(&build_context.build_path);
    copy_files_to_build_dir(&build_context, &build_path)?;
    build_context.write_requirements_file(
        service_path.join("requirements.txt").to_str().unwrap(),
        &lock_map,
    )?;
    run_docker_build(service_path, &docker_build_args)?;
    Ok(())
}

pub fn clean_bobpy_cache() -> Result<(), Box<dyn std::error::Error>> {
    let bobpy_build_path = PathBuf::from(".").join(".bobpy");
    fs::remove_dir_all(bobpy_build_path)?;
    Ok(())
}
