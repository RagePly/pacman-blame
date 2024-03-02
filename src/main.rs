use alpm::Alpm;
use std::env;
use std::error::Error;
use std::fmt;
use std::process::ExitCode;

mod argparse;
mod listing;
mod output;
mod query;

#[derive(Debug)]
enum ProgramError {
    NoPackagesFound,
    InvalidFormat(String),
    InvalidRequest(String),
    InvalidQuery(query::ParseError),
}

impl Error for ProgramError {}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use ProgramError::*;
        match self {
            NoPackagesFound => write!(f, "no matching packages found"),
            InvalidFormat(format) => write!(f, "invalid format '{format}'"),
            InvalidRequest(explination) => write!(f, "{}", explination),
            InvalidQuery(parse_error) => parse_error.fmt(f),
        }
    }
}

impl From<query::ParseError> for ProgramError {
    fn from(pe: query::ParseError) -> ProgramError {
        ProgramError::InvalidQuery(pe)
    }
}

fn print_helptext(option_text: String) {
    let lines = [
        "usage: pacman-blame [options] QUERY...".to_string(),
        "options:".to_string(),
        option_text,
        "".to_string(),
        "QUERY:".to_string(),
        "[package:]<package-name>  search the database for the exact name".to_string(),
        "".to_string(),
        "Use -h|--help after an option for more details".to_string(),
    ];

    println!("{}", lines.join("\n"));
}

fn main() -> ExitCode {
    let (api, common) = match argparse::parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{}", err);
            return ExitCode::from(1);
        }
    };

    match api {
        argparse::Api::Empty => println!("no command specified, use pacman-blame -h for help"),
        argparse::Api::Help => {
            print_helptext(argparse::print_argument_group(None).expect("None is valid group"))
        }
        argparse::Api::HelpWith(opt) => print_helptext(
            argparse::print_argument_group(Some(opt.as_str()))
                .expect("this should be supplied with a valid option"),
        ),
        argparse::Api::List(list) => {
            let Ok(handle) = Alpm::new("/", "/var/lib/pacman") else {
                eprintln!("could not connect to package database");
                return ExitCode::from(2);
            };
            match listing::list_packages(handle, list, common) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    return ExitCode::from(3);
                }
            }
        }
    }

    ExitCode::from(0)
}
