use super::commands::UserInput;
use super::file_changes::{self, FileChanges};
use super::{console_printer, dir_walker};
use std::io;
use std::path::{Path, PathBuf};

pub fn execute(user_input: UserInput) -> io::Result<()> {
    let init_path = Path::new(".");
    let file_paths =
        dir_walker::get_file_paths_that_match_expr(&user_input.pattern_string, &init_path)?;

    let changes_requested = file_changes::WantedChanges::from_user_input(&user_input);

    let mut files_seen = 0;
    let mut files_changed = 0;

    for file_path in file_paths.iter() {
        files_seen += 1;
        let possible_data =
            file_io::read_file_data_and_check_for_match(file_path, &changes_requested.old)?;
        if let Some(file_data) = possible_data {
            let changes_to_be_made = FileChanges::from_file_data(&file_data, &changes_requested);

            if !user_input.silent {
                console_printer::print_file_path_header_to_console(file_path);
                console_printer::print_changes_to_be_made(&changes_to_be_made);
            }

            if !user_input.dry_run {
                files_changed += 1;
                file_io::execute_changes_to_file(file_data, changes_to_be_made)?;
            }
        }
    }

    console_printer::print_current_counters(&files_seen, &files_changed);

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    mod dir_file_walker {
        use super::dir_walker::*;
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

            let some_lines = unwrap_and_check_ok(
                file_io::read_file_data_and_check_for_match(&path, statement_to_find),
                "reading file data for valid path should not return err",
            );

            assert!(some_lines.is_some());

            let lines = some_lines.unwrap();
            assert!(!lines.contents.is_empty());
            assert!(!lines.term_containing_lines.is_empty());
        }

        #[test]
        fn data_from_file_with_non_match_should_be_none() {
            let path = Path::new("Cargo.toml");
            let nonexistent_statement = "nonexistent_substring".repeat(10);

            let lines = unwrap_and_check_ok(
                file_io::read_file_data_and_check_for_match(&path, &nonexistent_statement),
                "reading file data for valid path should not return err",
            );

            assert!(lines.is_none());
        }
    }

    mod file_changes_tests {
        use super::file_changes::ParsedLine;
        use super::*;
        use ansi_term::Color;
        use std::collections::HashSet;

        fn mock_wanted_changes(old: &str, new: &str) -> file_changes::WantedChanges {
            file_changes::WantedChanges {
                old: old.to_string(),
                new: new.to_string(),
            }
        }

        #[test]
        fn should_be_able_to_create_from_file_data() {
            let changes_requested = mock_wanted_changes(" ", " ");
            let file_data = valid_file_data(&changes_requested.old);
            let changes = FileChanges::from_file_data(&file_data, &changes_requested);

            assert!(!changes.lines.is_empty());
        }

        #[test]
        fn number_of_lines_with_term_should_match() {
            // "[package]" is only present on one line in Cargo.toml by definition
            let changes_requested = mock_wanted_changes("[package]", " ");
            let file_data = valid_file_data(&changes_requested.old);
            let changes = FileChanges::from_file_data(&file_data, &changes_requested);

            assert!(!changes.lines.is_empty());
            assert_eq!(
                changes
                    .lines
                    .into_iter()
                    .filter(|line| line.has_term)
                    .collect::<Vec<ParsedLine>>()
                    .len(),
                1,
                "should be exactly one matching element in list"
            );
        }

        #[test]
        fn changes_should_represent_old_and_new_terms() {
            let old = "=";
            let new = "+";
            let changes_requested = mock_wanted_changes(&old, &new);
            let file_data = valid_file_data(&changes_requested.old);
            let changes = FileChanges::from_file_data(&file_data, &changes_requested);

            assert!(!changes.lines.is_empty());
            assert!(
                changes.lines.iter().all(|line| {
                    if let Some(new_term) = &line.contents.new.as_ref() {
                        let old_is_ok = line.contents.old.contains(&old);
                        let new_is_ok = new_term.1
                            == line
                                .contents
                                .old
                                .replace(&Color::Red.paint(old).to_string(), &new);
                        println!(
                            ";;;: {}, {}",
                            new_term.1,
                            line.contents.old.replace(&old, &new)
                        );
                        old_is_ok && new_is_ok
                    } else {
                        true
                    }
                }),
                "old and new terms from changes should match wanted changes"
            );
        }

        #[test]
        fn should_not_have_any_duplicate_lines() {
            let changes_requested = mock_wanted_changes(" ", " ");
            let file_data = valid_file_data(&changes_requested.old);
            let changes = FileChanges::from_file_data(&file_data, &changes_requested);

            assert!(!changes.lines.is_empty());
            let mut line_set = HashSet::new();
            let all_lines_inserted_non_dupe = changes
                .lines
                .into_iter()
                // HashSet.insert() returns false if no insert happened,
                // meaning that there was a duplicate entry.
                .all(|line| line_set.insert(line));
            assert!(all_lines_inserted_non_dupe);
        }
    }

    fn unwrap_and_check_ok<T>(result: io::Result<T>, assert_msg: &str) -> T {
        assert!(result.is_ok(), "{}", assert_msg);
        result.unwrap()
    }

    fn valid_file_data(term: &str) -> FileData {
        let path = Path::new("Cargo.toml");

        unwrap_and_check_ok(
            file_io::read_file_data_and_check_for_match(&path, term),
            "reading file data for valid path should not return err",
        )
        .expect("should not be none with valid path and term")
    }
}

pub struct FileData {
    pub file_path: PathBuf,
    pub contents: Vec<String>,
    pub term_containing_lines: Vec<usize>,
}
mod file_io {
    use super::{FileChanges, FileData};
    use std::fs::{self, File};
    use std::io::{self, BufRead};
    use std::path::Path;

    pub fn read_file_data_and_check_for_match(
        file_path: &Path,
        statement_to_find: &str,
    ) -> io::Result<Option<FileData>> {
        let file = File::open(file_path)?;
        let mut has_match = false;
        let mut contents = vec![];
        let mut term_containing_lines = vec![];

        let mut line_num = 0;
        for line in io::BufReader::new(file).lines().map(|x| x.unwrap()) {
            if line.contains(statement_to_find) {
                term_containing_lines.push(line_num);
                has_match = true;
            }
            contents.push(line);
            line_num += 1;
        }

        Ok(match has_match {
            false => None,
            true => Some(FileData {
                file_path: file_path.to_path_buf(),
                term_containing_lines,
                contents,
            }),
        })
    }

    pub fn execute_changes_to_file(
        mut file_data: FileData,
        changes: FileChanges,
    ) -> io::Result<()> {
        changes
            .lines
            .into_iter()
            .filter(|line| line.has_term)
            .map(|line| (line.num, line.contents))
            .for_each(|x| {
                let num = x.0;
                let replaced_line = x.1.new.unwrap();
                file_data.contents[num] = replaced_line.1;
            });

        // write file data
        let contents = file_data.contents.join("\n");
        fs::write(file_data.file_path, contents.as_bytes())?;
        Ok(())
    }
}
