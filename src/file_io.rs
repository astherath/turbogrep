use super::commands::UserInput;
use ansi_term::Color;
use glob;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

struct WantedChanges {
    old: String,
    new: String,
}

impl WantedChanges {
    fn from_user_input(user_input: &UserInput) -> Self {
        Self {
            old: user_input.old_term.to_string(),
            new: user_input.new_term.to_string(),
        }
    }
}

fn print_file_path_header_to_console(file_path: &Path) {
    let separator = "-".repeat(80);
    println!("\nFile: \"{:?}\"\n{}", &file_path, separator);
}

fn print_changes_to_be_made(changes_to_be_made: &FileChanges) {
    println!("{}", changes_to_be_made);
}

fn print_current_counters(files_seen: &u32, files_changed: &u32) {
    println!(
        "files seen: {}, files changed: {}...",
        files_seen, files_changed
    );
}

pub fn execute(user_input: UserInput) -> io::Result<()> {
    let init_path = Path::new(".");
    let file_paths = get_file_paths_that_match_expr(&user_input.pattern_string, &init_path)?;

    let changes_requested = WantedChanges::from_user_input(&user_input);

    let mut files_seen = 0;
    let mut files_changed = 0;

    for file_path in file_paths.iter() {
        files_seen += 1;
        let possible_data = read_file_data_and_check_for_match(file_path, &changes_requested.old)?;
        if let Some(file_data) = possible_data {
            let changes_to_be_made = FileChanges::from_file_data(&file_data, &changes_requested);

            if !user_input.silent {
                print_file_path_header_to_console(file_path);
                print_changes_to_be_made(&changes_to_be_made);
            }

            if !user_input.dry_run {
                files_changed += 1;
                execute_changes_to_file(file_data, changes_to_be_made)?;
            }
        }
    }

    print_current_counters(&files_seen, &files_changed);

    Ok(())
}

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

            let some_lines = unwrap_and_check_ok(
                read_file_data_and_check_for_match(&path, statement_to_find),
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
                read_file_data_and_check_for_match(&path, &nonexistent_statement),
                "reading file data for valid path should not return err",
            );

            assert!(lines.is_none());
        }
    }

    mod file_changes {
        use super::*;

        fn mock_wanted_changes(old: &str, new: &str) -> WantedChanges {
            WantedChanges {
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
        fn lines_should_be_sorted_by_line_num_desc() {
            let changes_requested = mock_wanted_changes("2018", "2021");
            let file_data = valid_file_data(&changes_requested.old);
            let changes = FileChanges::from_file_data(&file_data, &changes_requested);

            assert!(!changes.lines.is_empty());
            assert!(changes.lines.iter().is_sorted());
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
            read_file_data_and_check_for_match(&path, term),
            "reading file data for valid path should not return err",
        )
        .expect("should not be none with valid path and term")
    }
}

struct FileData {
    pub file_path: PathBuf,
    pub contents: Vec<String>,
    pub term_containing_lines: Vec<usize>,
}

fn read_file_data_and_check_for_match(
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

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
struct ParsedLine {
    pub num: usize,
    pub has_term: bool,
    pub contents: ChangeContents,
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
struct ChangeContents {
    old: String,
    new: Option<(String, String)>,
}

impl ChangeContents {
    fn from_line(line: &str, changes_requested: &WantedChanges, has_term: bool) -> Self {
        let old = &changes_requested.old;
        let new = &changes_requested.new;
        match has_term {
            true => Self {
                old: replace_terms_and_highlight(line, old, old, Color::Red),
                new: Some((
                    replace_terms_and_highlight(line, old, new, Color::Green),
                    line.replace(old, new),
                )),
            },
            false => Self {
                old: line.to_string(),
                new: None,
            },
        }
    }
}

impl fmt::Display for ChangeContents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_string = format!("{}", &self.old);
        if let Some(new) = &self.new {
            display_string.push_str(&format!(" -> {}", new.0))
        }
        write!(f, "{}", display_string)
    }
}

fn replace_terms_and_highlight(line: &str, old: &str, new: &str, highlight_color: Color) -> String {
    let colored_string = highlight_color.paint(new).to_string();
    line.replace(old, &colored_string)
}

impl Ord for ParsedLine {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num.cmp(&other.num)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct FileChanges {
    lines: Vec<ParsedLine>,
}

impl FileChanges {
    fn from_file_data(file_data: &FileData, changes_requested: &WantedChanges) -> Self {
        let old_term = &changes_requested.old;
        let mut line_set = HashSet::new();
        file_data.term_containing_lines.iter().for_each(|line_num| {
            let half_offset: usize = 2;
            let full_offset = (half_offset * 2) + 1;
            let start_index = line_num.checked_sub(half_offset).unwrap_or(0);
            // we only want to take a few lines surrounding the painted one
            let mut line_num = start_index;
            file_data
                .contents
                .iter()
                .skip(start_index)
                .take(full_offset)
                .for_each(|line| {
                    let has_term = line.contains(old_term);
                    let contents = ChangeContents::from_line(&line, changes_requested, has_term);
                    let num = line_num;
                    line_num += 1;
                    let parsed_line = ParsedLine {
                        has_term,
                        num,
                        contents,
                    };
                    line_set.insert(parsed_line);
                });
        });
        let mut lines = line_set.into_iter().collect::<Vec<ParsedLine>>();
        lines.sort_unstable();

        Self { lines }
    }
}

impl fmt::Display for ParsedLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>3}|  {}", self.num, self.contents)
    }
}

impl fmt::Display for FileChanges {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n",
            self.lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn execute_changes_to_file(mut file_data: FileData, changes: FileChanges) -> io::Result<()> {
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

fn clap_panic<T: fmt::Display>(details: T) -> ! {
    clap::Error::with_description(
        &format!("Error processing command. Details: {}", details),
        clap::ErrorKind::InvalidValue,
    )
    .exit()
}
