use std::{
    fs::{copy, create_dir_all},
    path::Path,
    process::{Command, Stdio},
};

pub(crate) fn run_command(command: &str, args: Vec<&str>) -> Option<()> {
    match Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
    {
        Ok(mut convert) => match convert.wait() {
            Ok(_) => Some(()),
            Err(why) => {
                eprintln!("Conversion failed: {why}");
                None
            }
        },
        Err(why) => {
            eprintln!("Conversion failed: {why}");
            None
        }
    }
}

/// Copy a file without altering it
pub(crate) fn copy_raw(from: &Path, to: &Path) {
    // Ensure the directory tree exists
    if let Some(folder) = to.parent() {
        if !folder.exists() {
            if let Err(why) = create_dir_all(folder) {
                eprintln!("Unable to create {folder:?}: {why}");
            }
        }
    }
    if let Err(why) = copy(from, to) {
        eprintln!("Unable to copy {from:?} to {to:?}: {why}");
    };
}
