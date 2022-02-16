mod cli;
mod console_printer;
mod errors;
mod file_parser;
mod parser_invoker;
fn main() {
    cli::run_main().expect("main cli fn failed");
}
