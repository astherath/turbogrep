use super::file_io::FileChanges;
use std::path::Path;

pub fn print_file_path_header_to_console(file_path: &Path) {
    let separator = "-".repeat(80);
    println!("\nFile: \"{:?}\"\n{}", &file_path, separator);
}

pub fn print_changes_to_be_made(changes_to_be_made: &FileChanges) {
    println!("{}", changes_to_be_made);
}

pub fn print_current_counters(files_seen: &u32, files_changed: &u32) {
    println!(
        "files seen: {}, files changed: {}...",
        files_seen, files_changed
    );
}
