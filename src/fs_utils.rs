use fs_extra::dir;
use std::fs;
use std::path::Path;

pub fn copy(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let todest = to.join(from);
    if from.is_file() {
        fs::create_dir_all(todest.parent().unwrap())?;
        fs::copy(from, todest)?;
    } else {
        dir::create_all(to, false)?;
        dir::copy(
            from,
            todest,
            &dir::CopyOptions {
                copy_inside: true,
                skip_exist: true,
                ..dir::CopyOptions::new()
            },
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;

    fn get_random_path() -> PathBuf {
        PathBuf::from(uuid::Uuid::new_v4().to_string())
    }

    #[test]
    fn test_copy_file_to_dir() {
        let from = get_random_path();
        fs::File::create(&from).unwrap();

        let todir = get_random_path().join(get_random_path());
        fs::create_dir_all(&todir).unwrap();
        let tofile = todir.join(&from);
        assert!(!tofile.exists());
        copy(&from, &todir).unwrap();
        assert!(tofile.exists());

        fs::remove_file(&from).unwrap();
        dir::remove(&todir.parent().unwrap()).unwrap();
    }

    #[test]
    fn test_copy_dir_to_dir() {
        let fromdir = get_random_path();
        fs::create_dir_all(&fromdir).unwrap();
        let fromfile = fromdir.join(get_random_path());
        fs::File::create(&fromfile).unwrap();
        let todir = get_random_path().join(get_random_path());
        let full_path = todir.join(&fromfile);
        fs::create_dir_all(&todir).unwrap();

        assert!(fromfile.exists() && fromfile.is_file());
        assert!(todir.exists() && todir.is_dir());
        assert!(!full_path.exists());
        copy(&fromdir, &todir).unwrap();
        assert!(full_path.exists() && full_path.is_file());

        dir::remove(fromdir).unwrap();
        dir::remove(todir.parent().unwrap()).unwrap();
    }
}
