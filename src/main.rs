use clap::{App, Arg, ArgMatches};

fn main() {
    println!("Hello, world!");
}

struct UserInput {
    regex_string: String,
    old_term: String,
    new_term: String,
}

trait ClapArg<'a, const I: usize> {
    const A: [&'a str; I];
    fn get_args<'b>() -> Vec<Arg<'a, 'b>>;
    fn from_matches(matches: &ArgMatches) -> Self;
}

impl<'a> ClapArg<'a, 2> for UserInput {
    const A: [&'a str; 2] = ["ab", "bc"];

    fn get_args<'b>() -> Vec<Arg<'a, 'b>> {
        vec![Arg::with_name("exp")]
    }

    fn from_matches(matches: &ArgMatches) -> Self {
        const REGEX_ARG_NAME: &str = "expr";
        const OLD_TERM_ARG_NAME: &str = "old";
        const NEW_TERM_ARG_NAME: &str = "new";
        Self {
            regex_string: matches.value_of(REGEX_ARG_NAME).unwrap().to_string(),
            old_term: matches.value_of(OLD_TERM_ARG_NAME).unwrap().to_string(),
            new_term: matches.value_of(NEW_TERM_ARG_NAME).unwrap().to_string(),
        }
    }
}
