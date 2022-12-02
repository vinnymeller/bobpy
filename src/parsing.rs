use crate::config::{RequirementName, RequirementLockMap};
use glob::glob_with;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, Debug)]
pub struct BuildFile {
    pub build_path: PathBuf,
    pub requirements: Vec<RequirementName>,
    pub libraries: Vec<PathBuf>,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct BuildContext {
    pub build_path: PathBuf,
    pub requirements: HashSet<RequirementName>,
    pub libraries: HashSet<PathBuf>,
    pub paths: HashSet<PathBuf>,
}

impl BuildContext {
    pub fn get_versioned_requirements(&self, lock_map: &RequirementLockMap) -> Vec<String> {
        self.requirements
            .iter()
            .map(|req| {
                let version = lock_map.get(req).expect(format!("Requirement {} not found in lock map", req).as_str());
                format!("{}{}", req, version)
            })
            .collect()
    }

    pub fn write_requirements_file(&self, path: &str, lock_map: &RequirementLockMap) -> Result<(), Box<dyn std::error::Error>> {
        let requirements = self.get_versioned_requirements(lock_map);
        let requirements_str = requirements.join("\n");
        std::fs::write(path, requirements_str)?;
        Ok(())
    }
}

pub fn parse_build_file(build_file_path: &Path) -> Result<BuildFile, config::ConfigError> {
    if !build_file_path.exists() {
        return Err(config::ConfigError::Message(
            "BUILD file does not exist".to_string(),
        ));
    }
    let build = config::Config::builder()
        .set_override("build_path", build_file_path.to_str())?
        .set_default("requirements", Vec::<RequirementName>::new())?
        .set_default("libraries", Vec::<String>::new())?
        .set_default("paths", Vec::<String>::new())?
        .add_source(config::File::new(build_file_path.to_str().unwrap(), config::FileFormat::Toml))
        .build()?;
    Ok(build.try_deserialize::<BuildFile>()?)
}

pub fn parse_build_file_recursively(build_file: &BuildFile) -> BuildContext {
    let mut context = BuildContext {
        build_path: build_file.build_path.parent().unwrap().to_path_buf(),
        requirements: build_file.requirements.iter().cloned().collect(),
        libraries: build_file.libraries.iter().cloned().collect(),
        paths: build_file.paths.iter().cloned().collect(),
    };

    let mut libs_to_check = build_file.libraries.clone();
    while libs_to_check.len() > 0 {
        let check_lib = libs_to_check.pop().unwrap();
        let build_path_glob = check_lib.to_str().unwrap().to_owned() + "/**/BUILD";
        // let build_paths = glob(&build_path_glob).unwrap();
        let build_paths = glob_with(
            &build_path_glob,
            glob::MatchOptions {
                case_sensitive: true,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        )
        .unwrap();
        for build_path in build_paths {
            println!("Found build file: {}", build_path.as_ref().unwrap().to_str().unwrap());
            // get string from pathbuf
            let temp_build_file = parse_build_file(build_path.unwrap().as_path()).unwrap();
            for path in temp_build_file.paths {
                context.paths.insert(path);
            }
            for lib in temp_build_file.libraries[..].iter() {
                if context.libraries.insert(lib.clone()) {
                    libs_to_check.push(lib.to_path_buf());
                }
            }
            for req in temp_build_file.requirements[..].iter() {
                context.requirements.insert(req.to_string());
            }
        }
    }

    context
}

pub fn get_build_context(service_path: &str) -> Result<BuildContext, Box<dyn std::error::Error>> {
    let build_file_path = Path::new(service_path).join("BUILD");
    let build_config = parse_build_file(Path::new(&build_file_path))?;
    Ok(parse_build_file_recursively(&build_config))
}
