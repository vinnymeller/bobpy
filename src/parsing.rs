use crate::config::{RequirementLockMap, RequirementName};
use glob::glob_with;
use std::collections::HashSet;
use std::io::{BufReader, Read};
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
                let version = lock_map
                    .get(req)
                    .unwrap_or_else(|| panic!("Requirement {} not found in lock map", req));
                format!("{}{}", req, version)
            })
            .collect()
    }

    pub fn write_requirements_file(
        &self,
        path: &str,
        lock_map: &RequirementLockMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let requirements = self.get_versioned_requirements(lock_map);
        let requirements_str = requirements.join("\n");
        std::fs::write(path, requirements_str)?;
        Ok(())
    }
}

pub fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn parse_build_file(build_file_path: &Path) -> Result<BuildFile, config::ConfigError> {
    let file_contents = match read_file(build_file_path) {
        Ok(contents) => contents,
        Err(e) => {
            return Err(config::ConfigError::Message(format!(
                "Error reading build file {}: {}",
                build_file_path.display(),
                e
            )))
        }
    };

    let build_file_config = get_build_file_config(build_file_path, &file_contents)?;
    Ok(build_file_config)
}

pub fn get_build_file_config(
    build_file_path: &Path,
    build_file_contents: &str,
) -> Result<BuildFile, config::ConfigError> {
    let config = config::Config::builder()
        .set_override("build_path", build_file_path.to_str())?
        .set_default("requirements", Vec::<RequirementName>::new())?
        .set_default("libraries", Vec::<String>::new())?
        .set_default("paths", Vec::<String>::new())?
        .add_source(config::File::from_str(
            build_file_contents,
            config::FileFormat::Toml,
        ))
        .build()?;
    let config = config.try_deserialize()?;
    Ok(config)
}

pub fn parse_build_file_recursively(build_file: &BuildFile) -> BuildContext {
    let mut context = BuildContext {
        build_path: build_file.build_path.parent().unwrap().to_path_buf(),
        requirements: build_file.requirements.iter().cloned().collect(),
        libraries: build_file.libraries.iter().cloned().collect(),
        paths: build_file.paths.iter().cloned().collect(),
    };

    let mut libs_to_check = build_file.libraries.to_owned();
    while !libs_to_check.is_empty() {
        let check_lib = libs_to_check.pop().unwrap().to_owned();
        let build_path_glob = format!("{}/**/BUILD", check_lib.display());
        println!("{}", build_path_glob);
        // let build_paths = glob(&build_path_glob).unwrap();
        let build_paths = glob_with(
            &build_path_glob,
            glob::MatchOptions {
                case_sensitive: true,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        )
        .unwrap_or_else(|_| panic!("Error parsing glob {}", build_path_glob));

        for build_path in build_paths {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_build_file_config_defaults() {
        let build_file_contents = "";
        let build_file_path = Path::new("test");
        let build_config = get_build_file_config(build_file_path, build_file_contents).unwrap();
        assert_eq!(build_config.build_path, build_file_path);
        assert_eq!(build_config.requirements, Vec::<RequirementName>::new());
        assert_eq!(build_config.libraries, Vec::<PathBuf>::new());
        assert_eq!(build_config.paths, Vec::<PathBuf>::new());
    }

    #[test]
    fn test_get_build_file_config_overrides() {
        let build_file_contents = r#"
            requirements = ["foo", "bar"]
            libraries = ["baz"]
            paths = ["qux"]
        "#;
        let build_file_path = Path::new("test");
        let build_config = get_build_file_config(build_file_path, build_file_contents).unwrap();
        assert_eq!(build_config.build_path, build_file_path);
        assert_eq!(build_config.requirements, vec!["foo", "bar"]);
        assert_eq!(build_config.libraries, vec![PathBuf::from("baz")]);
        assert_eq!(build_config.paths, vec![PathBuf::from("qux")]);
    }

    #[test]
    fn test_get_build_file_config_invalid() {
        let build_file_contents = r#"
            requirements = "foo"
        "#;
        let build_file_path = Path::new("test");
        let build_config = get_build_file_config(build_file_path, build_file_contents);
        assert!(build_config.is_err());
    }
}
