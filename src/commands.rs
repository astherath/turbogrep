use clap::{Arg, ArgMatches};

#[derive(Default, Debug)]
pub struct UserInput {
    pub regex_string: String,
    pub old_term: String,
    pub new_term: String,
}

pub trait ClapArg<'a, const I: usize> {
    const ARG_NAMES: [&'a str; I];
    fn get_args<'b>() -> Vec<Arg<'a, 'b>>;
    fn from_matches(matches: &ArgMatches) -> Self;
    fn get_setters() -> Vec<fn(Self, &str) -> Self>;
}

impl<'a> ClapArg<'a, 3> for UserInput {
    const ARG_NAMES: [&'a str; 3] = ["expr", "old", "new"];

    fn get_args<'b>() -> Vec<Arg<'a, 'b>> {
        // order not important here, so we can afford to not use the arg name array
        vec![
            Arg::with_name("expr")
                .help("the regex expression to match the files for")
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
        ]
    }

    fn from_matches(matches: &ArgMatches) -> Self {
        Self::get_setters().iter().zip(Self::ARG_NAMES.iter()).fold(
            Self::default(),
            |acc, setter_value_tuple| {
                let setter = setter_value_tuple.0;
                let arg_name = setter_value_tuple.1;
                let arg_value = matches.value_of(arg_name).unwrap();
                setter(acc, arg_value)
            },
        )
    }

    fn get_setters() -> Vec<fn(Self, &str) -> Self> {
        vec![
            |mut this, expr| {
                this.regex_string = expr.to_string();
                this
            },
            |mut this, old| {
                this.old_term = old.to_string();
                this
            },
            |mut this, new| {
                this.new_term = new.to_string();
                this
            },
        ]
    }
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
        UserInput::from_matches(&matches);
        true
    }

    #[test]
    fn valid_input_should_create_valid_matches() {
        let input = vec!["expr", "old", "new"];

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_ok());
        assert!(check_matches_valid(matches_result.unwrap()));
    }

    #[test]
    fn invalid_input_should_fail_match_parse() {
        let input = vec!["expr", "old", "new", "extra-arg"];

        let matches_result = get_matches_for_input(input);
        assert!(matches_result.is_err());
        assert_eq!(
            matches_result.err().unwrap().kind,
            ErrorKind::UnknownArgument
        );
    }
}
