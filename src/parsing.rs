use crate::config::{Requirement, RequirementName, BOBPY_CONFIG};
use crate::fs_utils;
use glob::glob_with;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct BuildDependencies {
    pub requirements: Vec<RequirementName>,
    pub libraries: Vec<PathBuf>,
    pub paths: Vec<PathBuf>,
}

impl BuildDependencies {
    pub fn from_str(
        build_dependencies_str: &str,
    ) -> Result<BuildDependencies, Box<dyn std::error::Error>> {
        let config = config::Config::builder()
            .set_default("requirements", Vec::<RequirementName>::new())?
            .set_default("libraries", Vec::<String>::new())?
            .set_default("paths", Vec::<String>::new())?
            .add_source(config::File::from_str(
                build_dependencies_str,
                config::FileFormat::Toml,
            ))
            .build()?;
        let config = config.try_deserialize()?;
        Ok(config)
    }
    pub fn from_path(path: &Path) -> Result<BuildDependencies, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let build_dependencies = BuildDependencies::from_str(&contents)?;
        Ok(build_dependencies)
    }
}

#[derive(Deserialize, Debug)]
pub struct BuildFile {
    pub path: PathBuf,
    pub dependencies: BuildDependencies,
}

impl BuildFile {
    pub fn from_path(path: &Path) -> Result<BuildFile, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let build_dependencies = BuildDependencies::from_str(&contents)?;
        Ok(BuildFile {
            path: path.parent().unwrap().to_path_buf(),
            dependencies: build_dependencies,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ServiceDependencies {
    pub requirements: HashSet<RequirementName>,
    pub libraries: HashSet<PathBuf>,
    pub paths: HashSet<PathBuf>,
}

impl ServiceDependencies {
    pub fn new() -> ServiceDependencies {
        ServiceDependencies {
            requirements: HashSet::new(),
            libraries: HashSet::new(),
            paths: HashSet::new(),
        }
    }

    pub fn extend(&mut self, build_file_dependencies: &BuildDependencies) {
        self.requirements
            .extend(build_file_dependencies.requirements.iter().cloned());
        self.libraries
            .extend(build_file_dependencies.libraries.iter().cloned());
        self.paths
            .extend(build_file_dependencies.paths.iter().cloned());
    }
}

#[derive(Deserialize, Debug)]
pub struct Service {
    pub path: PathBuf,
    pub dependencies: ServiceDependencies,
}

impl Service {
    pub fn from_path(path: &Path) -> Result<Service, Box<dyn std::error::Error>> {
        let build_file = BuildFile::from_path(&path.join("BUILD"))?;
        let mut service_dependencies = ServiceDependencies::new();
        service_dependencies.extend(&build_file.dependencies);
        let mut lib_paths_to_check = build_file.dependencies.libraries.to_owned();
        while !lib_paths_to_check.is_empty() {
            let lib_path = lib_paths_to_check.pop().unwrap();
            let build_file_glob = format!("{}/**/BUILD", lib_path.display());
            let build_file_paths_in_lib = glob_with(&build_file_glob, glob::MatchOptions::new())?;
            for lib_build_file_path in build_file_paths_in_lib {
                let lib_build_file_path = lib_build_file_path?;
                service_dependencies
                    .libraries
                    .insert(lib_build_file_path.parent().unwrap().to_path_buf());
                let lib_build_file = BuildFile::from_path(&lib_build_file_path)?;
                service_dependencies.extend(&lib_build_file.dependencies);
                for new_lib_to_check in lib_build_file.dependencies.libraries {
                    if !service_dependencies.libraries.contains(&new_lib_to_check) {
                        lib_paths_to_check.push(new_lib_to_check);
                    }
                }
            }
        }
        Ok(Service {
            path: path.to_path_buf(),
            dependencies: service_dependencies,
        })
    }

    fn get_versioned_requirements(&self) -> Vec<Requirement> {
        self.dependencies
            .requirements
            .iter()
            .map(|requirement_name| {
                let requirement_version =
                    BOBPY_CONFIG.requirement_lock.get(requirement_name).unwrap();
                Requirement {
                    name: requirement_name.to_owned(),
                    version: requirement_version.to_owned(),
                }
            })
            .collect()
    }

    fn get_requirements_list(&self) -> Vec<String> {
        let versioned_requirements = self.get_versioned_requirements();
        versioned_requirements
            .iter()
            .map(|requirement| requirement.to_string())
            .collect()
    }

    fn write_requirements_file(&self, build_path: &Path) -> Result<(), std::io::Error> {
        let mut requirements_list = self.get_requirements_list();
        requirements_list.sort();
        let requirements_file_path = build_path.join(&self.path).join("requirements.txt");
        fs::write(requirements_file_path, requirements_list.join("\n"))?;
        Ok(())
    }
    fn write_init_if_needed(&self, lib_path: &Path) -> Result<(), std::io::Error> {
        let init_file_path = lib_path.join("__init__.py");
        if !init_file_path.exists() {
            fs::write(init_file_path, "")?;
        }
        Ok(())
    }

    pub fn create_service_context(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_path = Path::new(".bobpy")
            .join("builds")
            .join(uuid::Uuid::new_v4().to_string());
        let mut paths_to_copy = Vec::new();
        paths_to_copy.push(&self.path);
        paths_to_copy.extend(self.dependencies.paths.iter());
        paths_to_copy.extend(self.dependencies.libraries.iter());
        fs::create_dir_all(&build_path)?;
        for path in paths_to_copy {
            fs_utils::copy(&path, &build_path.join(path))?;
        }
        self.write_requirements_file(&build_path)?;
        let build_libs_path = build_path.join(&BOBPY_CONFIG.project.libraries_path);
        self.write_init_if_needed(&build_libs_path)?;
        let lib_paths = glob_with(
            format!("{}/**", build_libs_path.to_str().unwrap()).as_str(),
            glob::MatchOptions::new())?;
        for lib_path in lib_paths {
            let lib_path = lib_path?;
            println!("lib_path: {}", lib_path.display());
            if lib_path.is_dir() {
                self.write_init_if_needed(&lib_path)?;
            }
        }

        Ok(())
    }
}
