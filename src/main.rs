use alpm::Alpm;
use std::env;
use std::error::Error;
use std::fmt;

mod argparse;
mod listing;

#[derive(Debug)]
enum ProgramError {
    PackageNotFound(String),
    Todo(String),
}

impl Error for ProgramError {}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use ProgramError::*;
        match self {
            PackageNotFound(opt) => write!(f, "package not found: {}", opt),
            Todo(explination) => write!(f, "todo encountered: {}", explination),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (api, common) = argparse::parse_args(env::args().skip(1))?;

    match api {
        argparse::Api::Empty => println!("no command specified, use pacman-blame -h for help"),
        argparse::Api::Help => println!(
            "{}",
            argparse::print_argument_group(None).expect("None is valid group")
        ),
        argparse::Api::HelpWith(opt) => println!(
            "{}",
            argparse::print_argument_group(Some(opt.as_str()))
                .expect("this should be supplied with a valid option")
        ),
        argparse::Api::List(list) => {
            let handle = Alpm::new("/", "/var/lib/pacman")?;
            listing::list_packages(handle, list, common)?;
        }
    }

    Ok(())
}
