use clap::{App, Arg, ArgMatches};

fn main() {
    let args = UserInput::get_args();
    let matches = args
        .into_iter()
        .fold(App::new("turbogrep"), |acc, arg| acc.arg(arg))
        .get_matches();
    let user_input = UserInput::from_matches(&matches);
    println!("user input: {:?}", user_input);
}

#[derive(Default, Debug)]
struct UserInput {
    regex_string: String,
    old_term: String,
    new_term: String,
}

trait ClapArg<'a, const I: usize> {
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
