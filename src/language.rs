// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

pub static VALID_LANGUAGES: [Language; 3] = [
    Language {
        name: "rust",
        artifact: "target/debug/intecture",
        runtime: None,
    },
    Language {
        name: "c",
        artifact: "intecture",
        runtime: None,
    },
    Language {
        name: "php",
        artifact: "bootstrap.php",
        runtime: Some("php"),
    },
];

#[derive(Debug)]
pub struct Language {
    pub name: &'static str,
    pub artifact: &'static str,
    pub runtime: Option<&'static str>,
}

impl Language {
    pub fn find(name: &str) -> Option<&'static Language> {
        for l in VALID_LANGUAGES.into_iter() {
            if l.name == name {
                return Some(l);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_nolang() {
        assert!(Language::find("NOLANG").is_none());
    }

    #[test]
    fn test_find_ok() {
        assert!(Language::find("rust").is_some());
    }
}
