use crate::commons::unwrap_and_check_ok;
use std::path::Path;

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
