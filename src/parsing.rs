use crate::config::BobpyConfig;
use crate::config::{Requirement, RequirementLockMap, RequirementName};
use crate::fs_utils;
use fs_extra::dir;
use glob::glob_with;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, PartialEq)]
pub struct BuildDependencies {
    pub requirements: Vec<RequirementName>,
    pub libraries: Vec<PathBuf>,
    pub paths: Vec<PathBuf>,
}

impl BuildDependencies {
    fn get_default_config_builder(
    ) -> Result<config::ConfigBuilder<config::builder::DefaultState>, config::ConfigError> {
        let builder = config::Config::builder()
            .set_default("requirements", Vec::<RequirementName>::new())?
            .set_default("libraries", Vec::<String>::new())?
            .set_default("paths", Vec::<String>::new())?;
        Ok(builder)
    }

    fn default() -> Result<BuildDependencies, config::ConfigError> {
        let config = Self::get_default_config_builder()?
            .build()?
            .try_deserialize()?;
        Ok(config)
    }

    fn from_str(
        build_dependencies_str: &str,
    ) -> Result<BuildDependencies, Box<dyn std::error::Error>> {
        let config = Self::get_default_config_builder()?
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
    pub fn default() -> ServiceDependencies {
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
pub struct ServiceContext {
    pub path: PathBuf,
    pub dependencies: ServiceDependencies,
}

impl ServiceContext {
    pub fn from_path(path: &Path) -> Result<ServiceContext, Box<dyn std::error::Error>> {
        let build_file = BuildFile::from_path(&path.join("BUILD"))?;
        let mut service_dependencies = ServiceDependencies::default();
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
        Ok(ServiceContext {
            path: path.to_path_buf(),
            dependencies: service_dependencies,
        })
    }

    pub fn get_all_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        paths.push(self.path.clone());
        paths.extend(self.dependencies.libraries.clone());
        paths.extend(self.dependencies.paths.clone());
        paths
    }

    fn get_versioned_requirements(
        &self,
        requirement_lock: &RequirementLockMap,
    ) -> Vec<Requirement> {
        self.dependencies
            .requirements
            .iter()
            .map(|requirement_name| {
                let requirement_version = requirement_lock.get(requirement_name).unwrap();
                Requirement {
                    name: requirement_name.to_owned(),
                    version: requirement_version.to_owned(),
                }
            })
            .collect()
    }

    fn get_requirements_list(&self, requirement_lock: &RequirementLockMap) -> Vec<String> {
        let versioned_requirements = self.get_versioned_requirements(requirement_lock);
        versioned_requirements
            .iter()
            .map(|requirement| requirement.to_string())
            .collect()
    }

    fn write_requirements_file(
        &self,
        build_path: &Path,
        requirement_lock: &RequirementLockMap,
    ) -> Result<(), std::io::Error> {
        let mut requirements_list = self.get_requirements_list(requirement_lock);
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

    pub fn write_service_context(
        &self,
        bobpy_config: &BobpyConfig,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let build_path = Path::new(".bobpy").join("builds").join(
            &self
                .path
                .strip_prefix(&bobpy_config.project.services_path)
                .unwrap(),
        );

        dir::create_all(&build_path, true)?;

        let mut paths_to_copy = Vec::new();
        paths_to_copy.push(&self.path);
        paths_to_copy.extend(self.dependencies.paths.iter());
        paths_to_copy.extend(self.dependencies.libraries.iter());
        fs::create_dir_all(&build_path)?;
        for path in paths_to_copy {
            fs_utils::copy(&path, &build_path)?;
        }
        self.write_requirements_file(&build_path, &bobpy_config.requirement_lock)?;
        let build_libs_path = build_path.join(&bobpy_config.project.libraries_path);
        self.write_init_if_needed(&build_libs_path)?;
        let lib_paths = glob_with(
            format!("{}/**", build_libs_path.to_str().unwrap()).as_str(),
            glob::MatchOptions::new(),
        )?;
        for lib_path in lib_paths {
            let lib_path = lib_path?;
            if lib_path.is_dir() {
                self.write_init_if_needed(&lib_path)?;
            }
        }

        Ok(PathBuf::from(build_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_dependencies_defaults_from_empty_str() {
        let build_dependencies = BuildDependencies::from_str("").unwrap();
        assert_eq!(build_dependencies.requirements.len(), 0);
        assert_eq!(build_dependencies.libraries.len(), 0);
        assert_eq!(build_dependencies.paths.len(), 0);
        assert_eq!(build_dependencies, BuildDependencies::default().unwrap());
    }

    #[test]
    fn test_build_dependencies_correctly_parses_inputs() {
        let build_dependencies = BuildDependencies::from_str(
            r#"
            requirements = ["requests", "flask"]
            libraries = ["lib1", "lib2"]
            paths = ["path1", "path2"]
            "#,
        )
        .unwrap();
        assert_eq!(build_dependencies.requirements, vec!["requests", "flask"]);
        assert_eq!(
            build_dependencies.libraries,
            vec![PathBuf::from("lib1"), PathBuf::from("lib2")]
        );
        assert_eq!(
            build_dependencies.paths,
            vec![PathBuf::from("path1"), PathBuf::from("path2")]
        );
    }

    #[test]
    fn test_build_dependencies_from_path() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let _ = BuildDependencies::from_path(file.path()).unwrap();
    }

    #[test]
    fn test_build_file_from_path() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let _ = BuildFile::from_path(file.path()).unwrap();
    }

    #[test]
    fn test_service_dependencies_extend() {
        let mut service_deps = ServiceDependencies::default();
        let build_deps = BuildDependencies {
            requirements: vec!["requests".to_string()],
            libraries: vec![PathBuf::from("lib1")],
            paths: vec![PathBuf::from("path1")],
        };

        service_deps.extend(&build_deps);
        assert!(service_deps.requirements.contains(&"requests".to_string()));
        assert!(service_deps.libraries.contains(&PathBuf::from("lib1")));
        assert!(service_deps.paths.contains(&PathBuf::from("path1")));
    }
}
