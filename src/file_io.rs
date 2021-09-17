use super::commands::UserInput;
use std::io;
use std::path::Path;

pub fn execute(user_input: UserInput) -> io::Result<()> {
    // read every file (one at a time) if it matches the regex

    let file_paths = get_file_paths_that_match_expr(&user_input.regex_string)?;

    // print (nicely) the name of the file and the prev/next
    file_paths.iter().for_each(|file_path| {
        let possible_data = read_file_data_and_check_for_match(file_path);
        if let Some(file_data) = possible_data {
            read_and_print_changes_to_be_made(&file_data);
            if !user_input.dry_run {
                execute_changes_to_file(&file_data);
            }
        }
    });

    // print line contents, line number, and strikethrough (in red) the old word and add next to it (green) the new word

    // if not --dry-run, then make the swap

    // return ok

    Ok(())
}

fn get_file_paths_that_match_expr(expr: &str) -> io::Result<Vec<&Path>> {
    Ok(vec![])
}

struct FileData;
fn read_file_data_and_check_for_match(file_path: &Path) -> Option<FileData> {
    None
}

fn read_and_print_changes_to_be_made(file_data: &FileData) -> io::Result<()> {
    Ok(())
}

fn execute_changes_to_file(file_data: &FileData) -> io::Result<()> {
    Ok(())
}
