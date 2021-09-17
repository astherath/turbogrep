use clap::App;
mod commands;
use commands::*;

fn main() {
    let args = UserInput::get_args();
    let matches = args
        .into_iter()
        .fold(App::new("turbogrep"), |acc, arg| acc.arg(arg))
        .get_matches();
    let user_input = UserInput::from_matches(&matches).unwrap();
    println!("user input: {:?}", user_input);
}
