use super::common;
use glob;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn get_file_paths_that_match_expr(
    expr: &str,
    starting_path: &Path,
) -> io::Result<Vec<PathBuf>> {
    let matches_pattern = |path: &Path| -> bool {
        let pattern = match glob::Pattern::new(expr) {
            Ok(re) => re,
            Err(error) => common::clap_panic(error),
        };
        pattern.matches_path(path)
    };

    let mut valid_paths = vec![];
    let mut add_file_to_list = |file_path: &fs::DirEntry| {
        if matches_pattern(&file_path.path()) {
            let full_path = file_path.path();
            valid_paths.push(full_path);
        }
    };

    fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&fs::DirEntry)) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
        Ok(())
    }

    visit_dirs(&starting_path, &mut add_file_to_list)?;

    Ok(valid_paths)
}
