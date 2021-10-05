use super::commands::UserInput;
use super::file_io::FileData;
use ansi_term::Color;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;
use std::fmt;

pub struct WantedChanges {
    pub old: String,
    pub new: String,
}

impl WantedChanges {
    pub fn from_user_input(user_input: &UserInput) -> Self {
        Self {
            old: user_input.old_term.to_string(),
            new: user_input.new_term.to_string(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
pub struct ParsedLine {
    pub num: usize,
    pub has_term: bool,
    pub contents: ChangeContents,
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
pub struct ChangeContents {
    pub old: String,
    pub new: Option<(String, String)>,
}

impl ChangeContents {
    pub fn from_line(line: &str, changes_requested: &WantedChanges, has_term: bool) -> Self {
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
pub struct FileChanges {
    pub lines: Vec<ParsedLine>,
}

impl FileChanges {
    pub fn from_file_data(file_data: &FileData, changes_requested: &WantedChanges) -> Self {
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
