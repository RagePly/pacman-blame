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
}

impl<S> PartialEq<S> for CliOption
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        self.long == other.as_ref()
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
    }
}

/*
const fn option_long(long: &'static str, comment: &'static str) -> CliOption {
    CliOption {
        short: None,
        long,
        comment,
        group: None,
    }
}

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

const OPT_HELP: CliOption = option("-h", "--help", "display on any item");
const OPT_VERBOSE: CliOption = option("-v", "--verbose", "print information of what is going on");
const OPT_API_LIST: CliOption = option("-L", "--list", "utilities for listing packages");
const OPT_API_LIST_EXPLICIT: CliOption = suboption(
    &OPT_API_LIST,
    "-e",
    "--explicit",
    "list only explicitly installed packages",
);
const OPT_API_LIST_DEPENDENCY: CliOption = suboption(
    &OPT_API_LIST,
    "-d",
    "--dependency",
    "list packages installed as dependencies",
);

const OPT_LIST: [CliOption; 5] = [
    OPT_HELP,
    OPT_VERBOSE,
    OPT_API_LIST,
    OPT_API_LIST_EXPLICIT,
    OPT_API_LIST_DEPENDENCY,
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
        if !option.starts_with("-") || option.starts_with("--") {
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
    pub packages: Vec<String>,
    pub explicit: bool,
    pub dependency: bool,
}

impl ApiList {
    fn new() -> ApiList {
        ApiList {
            packages: Vec::new(),
            explicit: false,
            dependency: false,
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
            opt if !is_option(&opt) => {
                self.packages.push(option);
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
}

impl Default for CommonOptions {
    fn default() -> CommonOptions {
        CommonOptions { verbose: false }
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
            opt if OPT_VERBOSE == opt => {
                if !self.common.verbose {
                    self.common.verbose = true;
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
                 ..
             }| {
                let option_symbol = if let Some(short) = short {
                    format!("{}|{}", short, long)
                } else {
                    format!("   {}", long)
                };
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
