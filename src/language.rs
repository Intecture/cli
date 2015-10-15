// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

pub static VALID_LANGUAGES: [Language; 4] = [
    Language {
        name: "rust",
        artifact: "target/debug/hello_world",
        runtime: None,
        example_repo: "https://github.com/betweenlines/intecture-example-rust.git",
    },
    Language {
        name: "c",
        artifact: "hello_world",
        runtime: None,
        example_repo: "https://github.com/betweenlines/intecture-example-c.git",
    },
    Language {
        name: "php",
        artifact: "bootstrap.php",
        runtime: Some("php"),
        example_repo: "https://github.com/betweenlines/intecture-example-php.git",
    },
    Language {
        name: "ruby",
        artifact: "bootstrap.rb",
        runtime: Some("ruby"),
        example_repo: "https://github.com/betweenlines/intecture-example-ruby.git",
    },
];

pub struct Language {
    pub name: &'static str,
    pub artifact: &'static str,
    pub runtime: Option<&'static str>,
    pub example_repo: &'static str,
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