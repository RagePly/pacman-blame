use alpm::Alpm;
use super::argparse::{ApiList as ListOptions, CommonOptions};
use super::ProgramError;

pub fn list_packages(
    handle: Alpm,
    ListOptions {
        packages,
        explicit,
        dependency,
    }: ListOptions,
    CommonOptions { verbose, .. }: CommonOptions,
) -> Result<(), ProgramError> {
   
    if verbose {
        println!(
            "will search for packages {} in explicit? {:?} in dependencies? {:?}",
            packages.join(", "), explicit, dependency
        );
    }
    
    Err(ProgramError::Todo("finish list_packages".to_string()))
}
