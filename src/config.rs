use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
pub struct ProjectConfig {
    pub services_path: PathBuf,
    pub libraries_path: PathBuf,
}

pub struct Requirement {
    pub name: RequirementName,
    pub version: RequirementVersion,
}

impl Requirement {
    pub fn to_string(&self) -> String {
        format!("{}=={}", self.name, self.version)
    }
}

pub type RequirementName = String;
pub type RequirementVersion = String;
pub type RequirementLockMap = HashMap<RequirementName, RequirementVersion>;

#[derive(serde::Deserialize, Debug)]
pub struct BobpyConfig {
    pub project: ProjectConfig,
    pub requirement_lock: RequirementLockMap,
}

impl BobpyConfig {
    pub fn load() -> Result<BobpyConfig, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(".bobpy.toml")?;
        let bobpy_config = BobpyConfig::from_str(&contents)?;
        Ok(bobpy_config)
    }
    pub fn from_str(config_str: &str) -> Result<BobpyConfig, config::ConfigError> {
        let settings = config::Config::builder()
            .set_default("project.services_path", "services")?
            .set_default("project.libraries_path", "libraries")?
            .set_default("requirement_lock", RequirementLockMap::new())?
            .add_source(config::File::from_str(config_str, config::FileFormat::Toml))
            .build()?;

        let bobpy_config = settings.try_deserialize()?;
        Ok(bobpy_config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_bobpy_config() {
        let contents = "";
        let bobpy_config = BobpyConfig::from_str(contents).unwrap();
        assert_eq!(
            bobpy_config.project.services_path,
            PathBuf::from("services")
        );
        assert_eq!(
            bobpy_config.project.libraries_path,
            PathBuf::from("libraries")
        );
        assert_eq!(bobpy_config.requirement_lock.len(), 0);
    }

    #[test]
    fn test_bobpy_settings_basic_overwrite() {
        let contents = r#"
            [project]
            services_path = "my_services"
            libraries_path = "my_libraries"

            [requirement_lock]
            "my_package" = "1.0.0"
            "my_other_package" = "2.0.0"
            "#;
        let bobpy_config = BobpyConfig::from_str(contents).unwrap();
        assert_eq!(
            bobpy_config.project.services_path,
            PathBuf::from("my_services")
        );
        assert_eq!(
            bobpy_config.project.libraries_path,
            PathBuf::from("my_libraries")
        );
        assert_eq!(bobpy_config.requirement_lock.len(), 2);
        assert_eq!(bobpy_config.requirement_lock["my_package"], "1.0.0");
        assert_eq!(bobpy_config.requirement_lock["my_other_package"], "2.0.0");
    }

    #[test]
    fn test_requirement_to_string() {
        let requirement = Requirement {
            name: "my_package".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(requirement.to_string(), "my_package==1.0.0");
    }
}
