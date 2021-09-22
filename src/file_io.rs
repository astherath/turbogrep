use super::commands::UserInput;
use glob;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub fn execute(user_input: UserInput) -> io::Result<()> {
    // read every file (one at a time) if it matches the pattern

    let init_path = Path::new(".");
    let file_paths = get_file_paths_that_match_expr(&user_input.pattern_string, &init_path)?;

    // print (nicely) the name of the file and the prev/next
    for file_path in file_paths.iter() {
        let possible_data = read_file_data_and_check_for_match(file_path, &user_input.old_term)?;
        if let Some(file_data) = possible_data {
            let changes_to_be_made = read_and_print_changes_to_be_made(&file_data)?;
            if !user_input.dry_run {
                execute_changes_to_file(&file_data, changes_to_be_made)?;
            }
        }
    }

    // print line contents, line number, and strikethrough (in red) the old word and add next to it (green) the new word

    // if not --dry-run, then make the swap

    // return ok

    Ok(())
}

struct FileChanges;

fn get_file_paths_that_match_expr(expr: &str, starting_path: &Path) -> io::Result<Vec<PathBuf>> {
    let matches_pattern = |path: &Path| -> bool {
        let pattern = match glob::Pattern::new(expr) {
            Ok(re) => re,
            Err(error) => clap_panic(error),
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

#[cfg(test)]
mod tests {
    use super::*;

    mod dir_file_walker {
        use super::*;

        #[test]
        fn should_read_files_from_src_dir() {
            let expr = &"*.rs";
            let init_path = Path::new(".");
            let files = unwrap_and_check_ok(
                get_file_paths_that_match_expr(expr, &init_path),
                "result from parsing dirs should not be err",
            );

            assert!(!files.is_empty());
        }

        #[test]
        fn dirs_should_have_correct_pattern_match() {
            let expr = &"*.rs";
            let init_path = Path::new(".");
            let files = unwrap_and_check_ok(
                get_file_paths_that_match_expr(expr, &init_path),
                "result from parsing dirs should not be err",
            );

            assert!(!files.is_empty());

            let pattern = glob::Pattern::new(expr).unwrap();
            files.into_iter().for_each(|path| {
                assert!(
                    pattern.matches_path(&path),
                    "paths of files received should match pattern"
                );
            });
        }
    }

    mod file_reader {
        use super::*;

        #[test]
        fn data_from_file_should_be_some() {
            let path = Path::new("Cargo.toml");
            let statement_to_find = " ";

            let lines = unwrap_and_check_ok(
                read_file_data_and_check_for_match(&path, statement_to_find),
                "reading file data for valid path should not return err",
            );

            assert!(lines.is_some());
            assert!(!lines.unwrap().contents.is_empty());
        }

        #[test]
        fn data_from_file_with_non_match_should_be_none() {
            let path = Path::new("Cargo.toml");
            let nonexistent_statement = "nonexistent_substring".repeat(10);

            let lines = unwrap_and_check_ok(
                read_file_data_and_check_for_match(&path, &nonexistent_statement),
                "reading file data for valid path should not return err",
            );

            assert!(lines.is_none());
        }
    }

    fn unwrap_and_check_ok<T>(result: io::Result<T>, assert_msg: &str) -> T {
        assert!(result.is_ok(), "{}", assert_msg);
        result.unwrap()
    }
}

struct FileData {
    pub file_path: PathBuf,
    pub contents: Vec<String>,
}

fn read_file_data_and_check_for_match(
    file_path: &Path,
    statement_to_find: &str,
) -> io::Result<Option<FileData>> {
    let file = File::open(file_path)?;
    let mut has_match = false;
    let mut contents = vec![];

    for line in io::BufReader::new(file).lines().map(|x| x.unwrap()) {
        // avoid needless `line.contains()` calls
        if !has_match && line.contains(statement_to_find) {
            has_match = true;
        }
        contents.push(line);
    }

    Ok(match has_match {
        true => Some(FileData {
            file_path: file_path.to_path_buf(),
            contents,
        }),
        false => None,
    })
}

fn read_and_print_changes_to_be_made(file_data: &FileData) -> io::Result<FileChanges> {
    Ok(FileChanges)
}

fn execute_changes_to_file(file_data: &FileData, changes: FileChanges) -> io::Result<()> {
    Ok(())
}

fn clap_panic<T: fmt::Display>(details: T) -> ! {
    clap::Error::with_description(
        &format!("Error processing command. Details: {}", details),
        clap::ErrorKind::InvalidValue,
    )
    .exit()
}
