// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod c;
mod php;
mod rust;

pub use self::c::CProject;
pub use self::php::PhpProject;
pub use self::rust::RustProject;
pub use inapi::Language;

use error::Result;
use std::{error, fmt};
use std::path::Path;
use std::process::ExitStatus;

pub fn language_from_str(lang: &str) -> Result<Language> {
    match lang {
        "c" => Ok(Language::C),
        "php" => Ok(Language::Php),
        "rust" => Ok(Language::Rust),
        _ => Err(LanguageError::UnknownLanguage(lang.into()).into()),
    }
}

pub trait LanguageProject {
    fn init_payload<P: AsRef<Path>>(path: P) -> Result<()>;
    fn init_project<P: AsRef<Path>>(path: P) -> Result<()>;
    fn run(args: &[&str]) -> Result<ExitStatus>;
}

#[derive(Debug)]
pub enum LanguageError {
    BuildFailed(String),
    CreateFailed(String),
    UnknownLanguage(String),
}

impl fmt::Display for LanguageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LanguageError::BuildFailed(ref e) => write!(f, "Could not build project: {}", e),
            LanguageError::CreateFailed(ref e) => write!(f, "Could not create language files for project: {}", e),
            LanguageError::UnknownLanguage(ref e) => write!(f, "Unknown language: {}", e),
        }
    }
}

impl error::Error for LanguageError {
    fn description(&self) -> &str {
        match *self {
            LanguageError::BuildFailed(_) => "Could not build project",
            LanguageError::CreateFailed(_) => "Could not create language files for project",
            LanguageError::UnknownLanguage(_) => "Unknown language",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert!(language_from_str("c").is_ok());
        assert!(language_from_str("php").is_ok());
        assert!(language_from_str("rust").is_ok());
        assert!(language_from_str("bang!").is_err());
    }
}
