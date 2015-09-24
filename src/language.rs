pub static VALID_LANGUAGES: [Language; 2] = [
    Language {
        name: "php",
        extension: "php",
        runtime: "php",
        bootstrap: "<?php

echo 'Hello world!';",
    },
    Language {
        name: "ruby",
        extension: "rb",
        runtime: "ruby",
        bootstrap: "puts \"Hello world!\";",
    },
];

pub struct Language {
    pub name: &'static str,
    pub extension: &'static str,
    pub runtime: &'static str,
    pub bootstrap: &'static str,
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