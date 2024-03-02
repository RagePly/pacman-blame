use super::argparse::{ApiList as ListOptions, CommonOptions};
use super::query::Query;
use super::ProgramError;
use super::output::CompiledFormat;
use alpm::{Alpm, PackageReason, Db, Pkg};
use std::collections::VecDeque;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ReasonSelector {
    Both,
    Explicit,
    Depend
}

trait Reason {
    fn is_explicit(&self) -> bool;
}

impl ReasonSelector {
    fn filter(self, reason: PackageReason) -> Option<PackageReason> {
        match (self, reason) {
            (ReasonSelector::Both, reason) => Some(reason),
            (ReasonSelector::Explicit, PackageReason::Explicit) => Some(reason),
            (ReasonSelector::Depend, PackageReason::Depend) => Some(reason),
            _ => None,
        }
    }
    
    fn test<R: Reason>(self, r: &R) -> bool {
        matches!(self, ReasonSelector::Both) || 
            matches!(self, ReasonSelector::Explicit) == r.is_explicit()
    }
    fn new(explicit: bool, dependency: bool) -> ReasonSelector {
        match (explicit, dependency) {
            (true, false) => ReasonSelector::Explicit,
            (false, true) => ReasonSelector::Depend,
            _ => ReasonSelector::Both,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ReqByItem {
    Explicit(String),
    Depend(String),
}

impl Reason for ReqByItem {
    fn is_explicit(&self) -> bool {
        matches!(self, ReqByItem::Explicit(_))
    }
}

impl ReqByItem {
    fn draw(self, color: bool) -> String {
        match self {
            ReqByItem::Explicit(name) if color => format!("\x1b[33m{}\x1b[m", name),
            ReqByItem::Explicit(name) | ReqByItem::Depend(name) => format!("{}", name), 
        }
    }
}

fn find_required_by<'h>(db: Db<'h>, pkg: Pkg<'h>, reason_filter: ReasonSelector) -> Vec<ReqByItem> {
    let mut queue: VecDeque<Pkg<'h>> = [pkg].into();
    let mut required_by: Vec<ReqByItem> = Vec::new();

    while !queue.is_empty() {
        let next = queue.pop_front().unwrap();
        let reqby = next.required_by();

        for name in reqby.iter().map(|s| s.to_string()) {
            let Ok(pkg) = db.pkg(name.clone()) else {
                eprintln!("failed to fetch info for {}", name);
                continue;
            };
            
            let reason = pkg.reason();
            let req = match reason {
                PackageReason::Explicit => ReqByItem::Explicit(name),
                PackageReason::Depend => ReqByItem::Depend(name),
            };

            if required_by.contains(&req) { 
                continue;
            }

            required_by.push(req);
            queue.push_back(*pkg);
        }
    }

    required_by.into_iter().filter(|r| reason_filter.test(r)).collect()
}

pub fn list_packages(
    handle: Alpm,
    ListOptions {
        queries,
        explicit,
        dependency,
        required_by,
    }: ListOptions,
    CommonOptions { color, format, .. }: CommonOptions,
) -> Result<(), ProgramError> {
    
    let compiled_format = match &format {
            Some(f) => CompiledFormat::compile(f.as_str()).ok_or(ProgramError::InvalidFormat(f.clone()))?,
            None => CompiledFormat::default(),
    };

    let number_queries = queries.len();
    
    if number_queries == 0 && required_by {
        return Err(ProgramError::InvalidRequest("you cannot use --required-by without specifying packages".to_string()));
    }

    let filter = ReasonSelector::new(explicit, dependency);

    let queries: Vec<Query> = queries
        .into_iter()
        .map(|s| Query::parse(&s))
        .collect::<Result<_, _>>()?;

    let local = handle.localdb();

    let pkgs: Vec<_> = if queries.is_empty() {
        local.pkgs().into_iter().collect()
    } else {
        queries
            .into_iter()
            .filter_map(|q| match q {
                Query::PackageName(name) => local.pkg(name).ok(),
            })
            .collect()
    };

    if pkgs.is_empty() {
        return Err(ProgramError::NoPackagesFound);
    }

    let mut lines: Vec<String> = Vec::new();
    for pkg in pkgs.into_iter() {
        if required_by {
            let reqby: Vec<_> = find_required_by(local, *pkg, filter).into_iter().map(|r| r.draw(color)).collect();
            if ! reqby.is_empty() {
                lines.push(reqby.join(" "));
            }
        } else if filter.filter(pkg.reason()).is_some() {
            lines.push(compiled_format.display(*pkg));
        }
    }

    if !lines.is_empty() {
        println!("{}", lines.join("\n"));
    }

    Ok(())
}
