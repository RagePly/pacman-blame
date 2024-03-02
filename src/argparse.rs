use std::cmp::PartialEq;
use std::default::Default;
use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
struct CliOption {
    pub short: Option<&'static str>,
    pub long: &'static str,
    pub comment: &'static str,
    pub group: Option<&'static CliOption>,
    pub takes_value: bool,
}

impl<S> PartialEq<S> for CliOption
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        if self.takes_value {
            let Some((option, _)) = other.as_ref().split_once("=") else {
                return false;
            };
            self.long == option
        } else {
            self.long == other.as_ref()
        }
    }
}

impl CliOption {
    fn is_subgroup(&self, other: &Self) -> bool {
        self.group.map(|g| g == other).unwrap_or(false)
    }
}

const fn option(short: &'static str, long: &'static str, comment: &'static str) -> CliOption {
    CliOption {
        short: Some(short),
        long,
        comment,
        group: None,
        takes_value: false,
    }
}

const fn suboption(
    group: &'static CliOption,
    short: &'static str,
    long: &'static str,
    comment: &'static str,
) -> CliOption {
    CliOption {
        short: Some(short),
        long,
        comment,
        group: Some(group),
        takes_value: false
    }
}

/*
const fn suboption_long(
    group: &'static CliOption,
    long: &'static str,
    comment: &'static str,
) -> CliOption {
    CliOption {
        short: None,
        long,
        comment,
        group: Some(group),
    }
}
*/

const fn option_long_value(long: &'static str, comment: &'static str) -> CliOption {
    CliOption {
        short: None,
        long,
        comment,
        group: None,
        takes_value: true,
    }
}


const OPT_HELP: CliOption = option("-h", "--help", "display on any item");
const OPT_VERBOSE: CliOption = option("-v", "--verbose", "print information of what is going on");
const OPT_COLOR: CliOption = option("-c", "--color", "use colors on terminals that support them");
const OPT_FORMAT: CliOption = option_long_value("--format", "print using the format");
const OPT_API_LIST: CliOption = option("-L", "--list", "utilities for listing packages");
const OPT_API_LIST_REQUIRED_BY: CliOption = suboption(
    &OPT_API_LIST,
    "-r", 
    "--required-by", 
    "show packages that requires this package"
);

const OPT_API_LIST_EXPLICIT: CliOption = suboption(
    &OPT_API_LIST,
    "-e",
    "--explicit",
    "filter on installed packages",
);
const OPT_API_LIST_DEPENDENCY: CliOption = suboption(
    &OPT_API_LIST,
    "-d",
    "--dependency",
    "filter on packages installed as a dependency",
);

const OPT_LIST: [CliOption; 8] = [
    OPT_COLOR,
    OPT_HELP,
    OPT_VERBOSE,
    OPT_FORMAT,
    OPT_API_LIST,
    OPT_API_LIST_EXPLICIT,
    OPT_API_LIST_DEPENDENCY,
    OPT_API_LIST_REQUIRED_BY,
];

fn is_option<S: AsRef<str>>(option: &S) -> bool {
    option.as_ref().starts_with("-")
}

fn is_long_option<S: AsRef<str>>(option: &S) -> bool {
    option.as_ref().starts_with("--")
}

fn is_short_option<S: AsRef<str>>(option: &S) -> bool {
    is_option(option) && !is_long_option(option)
}

fn split_short<I: IntoIterator<Item = String>>(options: I) -> impl IntoIterator<Item = String> {
    options
        .into_iter()
        .map(|option| {
            if !is_short_option(&option) {
                vec![option].into_iter()
            } else {
                option
                    .chars()
                    .skip(1)
                    .map(|c| format!("-{}", c))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
        })
        .flatten()
}

fn expand_short<I: IntoIterator<Item = String>>(
    options: I,
) -> impl IntoIterator<Item = Result<String, ArgError>> {
    options.into_iter().map(|option| {
        if !is_short_option(&option) {
            Ok(option)
        } else {
            OPT_LIST
                .iter()
                .filter_map(|CliOption { short, long, .. }| {
                    if short.map(|s| s == option).unwrap_or(false) {
                        Some(long.to_string())
                    } else {
                        None
                    }
                })
                .next()
                .ok_or(ArgError::UnknownOption(option))
        }
    })
}

#[derive(Debug)]
pub struct ApiList {
    pub queries: Vec<String>,
    pub explicit: bool,
    pub dependency: bool,
    pub required_by: bool,
}

impl ApiList {
    fn new() -> ApiList {
        ApiList {
            queries: Vec::new(),
            explicit: false,
            dependency: false,
            required_by: false,
        }
    }
    fn add_option(mut self, option: String) -> Result<Api, ArgError> {
        match option.as_str() {
            opt if OPT_HELP == opt => Ok(Api::HelpWith(OPT_API_LIST.long.to_string())),
            opt if OPT_API_LIST_EXPLICIT == opt => {
                if !self.explicit {
                    self.explicit = true;
                    Ok(Api::List(self))
                } else {
                    Err(ArgError::DuplicateOption(option))
                }
            }
            opt if OPT_API_LIST_DEPENDENCY == opt => {
                if !self.dependency {
                    self.dependency = true;
                    Ok(Api::List(self))
                } else {
                    Err(ArgError::DuplicateOption(option))
                }
            }
            opt if OPT_API_LIST_REQUIRED_BY == opt => {
                if !self.required_by {
                    self.required_by = true;
                    Ok(Api::List(self))
                } else {
                    Err(ArgError::DuplicateOption(option))
                }
            }
            opt if !is_option(&opt) => {
                self.queries.push(option);
                Ok(Api::List(self))
            }
            _ => Err(ArgError::UnknownOption(option)),
        }
    }

    fn apply_defaults(mut self) -> Self {
        if self.explicit == self.dependency {
            self.explicit = true;
            self.dependency = true;
        }

        self
    }
}

#[derive(Debug)]
pub enum Api {
    Empty,
    Help,
    HelpWith(String),
    List(ApiList),
}

impl Api {
    fn add_option(self, opt: String) -> Result<Self, ArgError> {
        match self {
            Api::Help => Ok(self),
            Api::HelpWith(_) => Ok(self),
            Api::Empty => match opt.as_str() {
                opt if OPT_API_LIST == opt => Ok(Api::List(ApiList::new())),
                opt if OPT_HELP == opt => Ok(Api::Help),
                unknown => Err(ArgError::UnknownOption(unknown.to_string())),
            },
            Api::List(list) => list.add_option(opt),
        }
    }

    fn apply_defaults(self) -> Self {
        match self {
            Api::List(list) => Api::List(list.apply_defaults()),
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct CommonOptions {
    pub verbose: bool,
    pub color: bool,
    pub format: Option<String>,
}

impl Default for CommonOptions {
    fn default() -> CommonOptions {
        CommonOptions { 
            verbose: false,
            color: false,
            format: None,
        }
    }
}

struct CliOptions {
    pub api: Api,
    pub common: CommonOptions,
}

impl CliOptions {
    fn new() -> CliOptions {
        CliOptions {
            api: Api::Empty,
            common: CommonOptions::default(),
        }
    }

    fn add_option(mut self, option: String) -> Result<Self, ArgError> {
        match option.as_str() {
            opt if OPT_FORMAT == opt => {
                let (prefix, value) = opt.split_once("=").expect("this has already been verified");
                if self.common.format.is_none() {
                    self.common.format = Some(value.to_string());
                    Ok(self)
                } else {
                    Err(ArgError::DuplicateOption(prefix.to_string()))
                }
            }
            opt if OPT_VERBOSE == opt => {
                if !self.common.verbose {
                    self.common.verbose = true;
                    Ok(self)
                } else {
                    Err(ArgError::DuplicateOption(option))
                }
            }
            opt if OPT_COLOR == opt => {
                if !self.common.color {
                    self.common.color = true;
                    Ok(self)
                } else {
                    Err(ArgError::DuplicateOption(option))
                }
            }
            _ => {
                self.api = self.api.add_option(option)?;
                Ok(self)
            }
        }
    }

    fn apply_defaults(mut self) -> Self {
        self.api = self.api.apply_defaults();
        self
    }
}

#[derive(Debug)]
pub enum ArgError {
    UnknownOption(String),
    DuplicateOption(String),
}

impl error::Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use ArgError::*;
        match self {
            UnknownOption(opt) => write!(f, "unknown option: {}", opt),
            DuplicateOption(opt) => write!(f, "duplicate option: {}", opt),
        }
    }
}

pub fn parse_args<I: IntoIterator<Item = String>>(
    args: I,
) -> Result<(Api, CommonOptions), ArgError> {
    expand_short(split_short(args))
        .into_iter()
        .fold(Ok(CliOptions::new()), |res, opt| {
            res.and_then(|cli| opt.and_then(|o| cli.add_option(o)))
        })
        .map(CliOptions::apply_defaults)
        .map(|CliOptions { api, common }| (api, common))
}

pub fn print_argument_group(option: Option<&str>) -> Result<String, ArgError> {
    // Get the actual group, unless root (None)
    let group = option
        .map(|option| {
            OPT_LIST
                .iter()
                .filter(|opt| opt.short.map(|s| s == option).unwrap_or(false) || opt.long == option)
                .next()
                .ok_or(ArgError::UnknownOption(option.to_string()))
        })
        .transpose()?;

    // Get all subgroups
    let mut subgroups: Vec<_> = OPT_LIST
        .iter()
        .filter(|opt| {
            group
                .map(|group| opt.is_subgroup(group))
                .unwrap_or(opt.group.is_none())
        })
        .collect();

    subgroups.sort_by(|g1, g2| {
        let lhs = g1.short.unwrap_or(g1.long);
        let rhs = g2.short.unwrap_or(g2.long);
        lhs.cmp(rhs)
    });

    let mut max_width = 0usize;
    let option_rows: Vec<_> = subgroups
        .into_iter()
        .map(
            |CliOption {
                 short,
                 long,
                 comment,
                 takes_value,
                 ..
             }| {
                let mut option_symbol = if let Some(short) = short {
                    format!("{}|{}", short, long)
                } else {
                    format!("   {}", long)
                };
                if *takes_value {
                    option_symbol.push_str("=VALUE");
                }
                max_width = max_width.max(option_symbol.len());
                (option_symbol, comment)
            },
        )
        .collect();

    let lines: Vec<_> = option_rows
        .into_iter()
        .map(|(opt, com)| format!("\t{opt:max_width$}\t{com}"))
        .collect();

    Ok(lines.join("\n"))
}
