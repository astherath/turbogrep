use std::io;

pub fn unwrap_and_check_ok<T>(result: io::Result<T>, assert_msg: &str) -> T {
    assert!(result.is_ok(), "{}", assert_msg);
    result.unwrap()
}
