use clap::App;
mod commands;
mod common;
mod console_printer;
mod dir_walker;
pub use commands::{ClapArg, UserInput};
mod file_io;

fn main() {
    let args = UserInput::get_args();
    let matches = args
        .into_iter()
        .fold(App::new("turbogrep"), |acc, arg| acc.arg(arg))
        .get_matches();
    let user_input = UserInput::from_matches(&matches).unwrap();
    file_io::execute(user_input).unwrap();
}
