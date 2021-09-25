use clap::{self, Arg, ArgMatches};

type ParseResult<T> = Result<T, ()>;

#[derive(Default, Debug)]
pub struct UserInput {
    pub pattern_string: String,
    pub old_term: String,
    pub new_term: String,
    pub dry_run: bool,
    pub silent: bool,
}

pub trait ClapArg<'a> {
    fn get_args<'b>() -> Vec<Arg<'a, 'b>>;
    fn from_matches(matches: &ArgMatches) -> ParseResult<Self>
    where
        Self: Sized;
    fn get_setters() -> Vec<fn(Self, &ArgMatches) -> Self>;
}

impl<'a> ClapArg<'a> for UserInput {
    fn get_args<'b>() -> Vec<Arg<'a, 'b>> {
        vec![
            Arg::with_name("expr")
                .help("the pattern expression to match the files for")
                .required(true)
                .takes_value(true)
                .index(1),
            Arg::with_name("old")
                .help("the (old) term currently present in the files to replace")
                .required(true)
                .takes_value(true)
                .index(2),
            Arg::with_name("new")
                .help("the (new) term to replace the old term with")
                .required(true)
                .takes_value(true)
                .index(3),
            Arg::with_name("dry-run")
                .help("if set, does not execute the final step of replacing the matching terms in the files")
                .long("dry-run")
                .short("d")
                .multiple(false)
                .required(false)
                ,
            Arg::with_name("silent")
                .help("if set, does not print out any output except the final files seen/changed count")
                .long("silent")
                .short("s")
                .multiple(false)
                .required(false)
        ]
    }

    fn from_matches(matches: &ArgMatches) -> ParseResult<Self> {
        Ok(Self::get_setters()
            .iter()
            .fold(Self::default(), |acc, setter| setter(acc, matches)))
    }

    fn get_setters() -> Vec<fn(Self, &ArgMatches) -> Self> {
        vec![
            |mut this, matches| {
                let arg_name = "expr";
                this.pattern_string = matches
                    .value_of(arg_name)
                    .ok_or_else(|| panic_because_of_bad_parse())
                    .unwrap()
                    .to_string();
                this
            },
            |mut this, matches| {
                let arg_name = "old";
                this.old_term = matches
                    .value_of(arg_name)
                    .ok_or_else(|| panic_because_of_bad_parse())
                    .unwrap()
                    .to_string();
                this
            },
            |mut this, matches| {
                let arg_name = "new";
                this.new_term = matches
                    .value_of(arg_name)
                    .ok_or_else(|| panic_because_of_bad_parse())
                    .unwrap()
                    .to_string();
                this
            },
            |mut this, matches| {
                let arg_name = "dry-run";
                this.dry_run = matches.is_present(arg_name);
                this
            },
            |mut this, matches| {
                let arg_name = "silent";
                this.silent = matches.is_present(arg_name);
                this
            },
        ]
    }
}

fn panic_because_of_bad_parse() -> ! {
    clap::Error::with_description(
        &"Command could not be parsed or was not passed in.",
        clap::ErrorKind::ArgumentNotFound,
    )
    .exit()
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::{App, AppSettings, ErrorKind, Result as ClapResult};
    fn get_matches_for_input(input: Vec<&str>) -> ClapResult<ArgMatches> {
        UserInput::get_args()
            .into_iter()
            .fold(
                App::new("turbogrep").setting(AppSettings::NoBinaryName),
                |acc, arg| acc.arg(arg),
            )
            .get_matches_from_safe(input)
    }

    fn check_matches_valid(matches: ArgMatches) -> bool {
        UserInput::from_matches(&matches).is_ok()
    }

    fn get_required_input_arg_values<'a>() -> Vec<&'a str> {
        vec!["expr", "old", "new"]
    }

    #[test]
    fn valid_input_should_create_valid_matches() {
        let input = get_required_input_arg_values();

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_ok());
        assert!(check_matches_valid(matches_result.unwrap()));
    }

    #[test]
    fn optional_dry_run_arg_should_work() {
        let mut input = get_required_input_arg_values();
        input.push("--dry-run");

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_ok());
        assert!(check_matches_valid(matches_result.unwrap()));
    }

    #[test]
    fn optional_silent_flag_should_work() {
        let mut input = get_required_input_arg_values();
        input.push("--silent");

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_ok());

        let user_input = UserInput::from_matches(&matches_result.unwrap()).unwrap();
        assert!(!user_input.dry_run);
        assert!(user_input.silent);
    }

    #[test]
    fn silent_flag_should_not_conflict_with_dry_run_flag() {
        let mut input = get_required_input_arg_values();
        input.push("--dry-run");
        input.push("--silent");

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_ok());
        let user_input = UserInput::from_matches(&matches_result.unwrap()).unwrap();
        assert!(user_input.dry_run);
        assert!(user_input.silent);
    }

    #[test]
    fn invalid_input_should_fail_match_parse() {
        let mut input = get_required_input_arg_values();
        input.push("extra-arg");

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_err());
        assert_eq!(
            matches_result.err().unwrap().kind,
            ErrorKind::UnknownArgument
        );
    }
}
