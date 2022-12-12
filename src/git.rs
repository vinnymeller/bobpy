use std::path::PathBuf;


pub fn get_changed_git_paths(base_branch: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(base_branch)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let changed_files = stdout
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    Ok(changed_files)
}

pub fn any_service_files_changed(
    changed_git_paths: &Vec<String>,
    service_paths: &Vec<PathBuf>,
) -> bool {
    // loop through all changed git files and see if the file path starts with any of the service paths
    for changed_git_path in changed_git_paths {
        for service_path in service_paths {
            if changed_git_path.starts_with(service_path.to_str().unwrap()) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_any_service_files_changed_gets_simple_change() {
        let changed_files = vec!["testdir/file1.txt".to_string()];
        let service_files = vec![PathBuf::from("testdir")];
        assert!(any_service_files_changed(&changed_files, &service_files));
    }

    #[test]
    fn test_any_service_files_changed_gets_no_change() {
        let changed_files = vec!["testdir/file1.txt".to_string()];
        let service_files = vec![PathBuf::from("testdir2")];
        assert!(!any_service_files_changed(&changed_files, &service_files));
    }

    #[test]
    fn test_any_service_files_changed_gets_change_in_nested_dirs() {
        let changed_files = vec!["testdir/dir1/file1.txt".to_string()];
        let service_files = vec![PathBuf::from("testdir")];
        assert!(any_service_files_changed(&changed_files, &service_files));
    }
}
