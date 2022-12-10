use std::path::Path;
use fs_extra::dir;
use std::fs;

pub fn copy(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
