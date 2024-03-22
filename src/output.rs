use alpm::{PackageReason, Pkg};
use std::default::Default;

enum Format<'a> {
    Text(&'a str),
    Name,
    Summary,
    Reason,
    Version,
}

enum ParseStatus<'a> {
    Invalid,
    NeedMore,
    Correct(Format<'a>),
}

impl<'a> Format<'a> {
    fn parse_token_slice(tokens: &'a str) -> ParseStatus<'a> {
        match tokens {
            "" => ParseStatus::NeedMore,
            "%" => ParseStatus::NeedMore,
            "%%" => ParseStatus::Correct(Format::Text(&tokens[0..1])),
            "%n" | "%{n}" => ParseStatus::Correct(Format::Name),
            "%s" | "%{s}" => ParseStatus::Correct(Format::Summary),
            "%r" | "%{r}" => ParseStatus::Correct(Format::Reason),
            "%v" | "%{v}" => ParseStatus::Correct(Format::Version),
            "%{" => ParseStatus::NeedMore,
            s if s.starts_with("%{") => {
                if s.ends_with("}") {
                    ParseStatus::Invalid
                } else {
                    ParseStatus::NeedMore
                }
            }
            s if s.starts_with("%") => ParseStatus::Invalid,
            s => ParseStatus::Correct(Format::Text(s)),
        }
    }
}

pub struct CompiledFormat<'a>(Vec<Format<'a>>);

impl<'a> CompiledFormat<'a> {
    pub fn compile(text: &'a str) -> Option<Self> {
        let mut start = 0;
        let mut end = 0;
        let mut format_parts = Vec::new();
        while start < text.len() {
            let chunk = &text[start..end];
            match Format::parse_token_slice(chunk) {
                ParseStatus::NeedMore => {
                    end += 1;
                    if end > text.len() {
                        return None;
                    }
                }
                ParseStatus::Invalid => {
                    return None;
                }
                ParseStatus::Correct(form) => {
                    format_parts.push(form);
                    start = end;
                    end += 1;
                }
            }
        }
        Some(CompiledFormat(format_parts))
    }

    pub fn display(&self, pkg: &Pkg) -> String {
        let mut output = String::new();
        self.0.iter().for_each(|part| match part {
            Format::Text(s) => output.push_str(s),
            Format::Name => output.push_str(pkg.name()),
            Format::Summary => output.push_str(pkg.desc().unwrap_or("")),
            Format::Reason => match pkg.reason() {
                PackageReason::Explicit => output.push_str("Explicit"),
                PackageReason::Depend => output.push_str("Depend"),
            },
            Format::Version => output.push_str(pkg.version().as_str()),
        });
        output
    }
}

impl Default for CompiledFormat<'static> {
    fn default() -> Self {
        CompiledFormat(vec![Format::Name])
    }
}
