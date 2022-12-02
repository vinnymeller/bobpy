use std::path::PathBuf;
use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct ProjectConfig {
    pub services_path: PathBuf,
    pub libraries_path: PathBuf,
}


pub type RequirementName = String;
pub type RequirementVersionLock = String;
pub type RequirementLockMap = HashMap<RequirementName, RequirementVersionLock>;

#[derive(serde::Deserialize, Debug)]
pub struct BobpyConfig {
    pub project: ProjectConfig,
    pub requirement_lock: RequirementLockMap,
}
#[derive(serde::Deserialize, Debug)]
pub struct BuildFile {
    pub requirements: Vec<String>,
    pub libraries: Vec<PathBuf>,
    pub paths: Vec<PathBuf>,
}


pub fn get_settings() -> Result<BobpyConfig, config::ConfigError> {
    let settings = config::Config::builder()
        .set_default("project.services_path", "services")?
        .set_default("project.libraries_path", "libraries")?
        .set_default("requirement_lock", RequirementLockMap::new())?
        .add_source(config::File::from(PathBuf::from(".bobpy.toml")))
        .build()?;

    Ok(settings.try_deserialize::<BobpyConfig>()?)
}


