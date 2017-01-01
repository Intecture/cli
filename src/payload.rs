// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::Result;
use inapi;
use language::{Language, LanguageProject, CProject, PhpProject, RustProject};
use std::{error, fmt, fs};
use std::path::{Path, PathBuf};
use std::process::Command;
use zdaemon::ConfigFile;

pub struct Payload;

impl Payload {
    pub fn find<P: AsRef<Path>>(payload_path: P, names: Option<&[&str]>) -> Result<Vec<inapi::Payload>> {
        let mut path = PathBuf::from("payloads");
        if payload_path.as_ref() != Path::new(".") {
            path.push(payload_path);
        }

        let mut payloads = Vec::new();

        if let Some(n) = names {
            for name in n {
                {
                    let payload_artifact = if path.is_relative() {
                        name
                    } else {
                        path.push(name);
                        path.to_str().unwrap()
                    };

                    payloads.push(try!(inapi::Payload::new(payload_artifact)));
                }

                if path.is_absolute() {
                    path.pop();
                }
            }
        } else {
            for entry in try!(fs::read_dir(&path)) {
                let entry = try!(entry);
                let dirname = entry.file_name().into_string().unwrap();
                path.push(&dirname);

                if path.is_dir() {
                    let payload = if path.is_relative() {
                        try!(inapi::Payload::new(&dirname))
                    } else {
                        try!(inapi::Payload::new(path.to_str().unwrap()))
                    };

                    payloads.push(payload);
                }

                path.pop();
            }
        }

        Ok(payloads)
    }

    pub fn create<P: AsRef<Path>>(payload_path: P, language: Language) -> Result<inapi::Payload> {
        let mut path = PathBuf::from("payloads");
        path.push(&payload_path);

        // Make sure folder doesn't already exist
        if path.exists() {
            return Err(PayloadError::PayloadExists.into());
        }

        path.push("data");
        try!(fs::create_dir_all(&path));
        path.pop();

        path.push("tpl");
        try!(fs::create_dir(&path));
        path.pop();

        // Create git repo
        let output = try!(Command::new("git").args(&["init", path.to_str().unwrap()]).output());
        if !output.status.success() {
            return Err(PayloadError::CreateFailed(try!(String::from_utf8(output.stderr))).into());
        }

        match language {
            Language::C => try!(CProject::init_payload(&path)),
            Language::Php => try!(PhpProject::init_payload(&path)),
            Language::Rust => try!(RustProject::init_payload(&path)),
        }

        // Create payload.json
        let dirname = try!(try!(path.file_name().ok_or(PayloadError::InvalidPath))
                                    .to_str().ok_or(PayloadError::InvalidPath)).to_owned();
        path.push("payload.json");
        let project_conf = inapi::PayloadConfig {
            author: "me".into(),
            repository: format!("https://github.com/<ORG>/{}.git", dirname),
            language: language,
            dependencies: None,
        };
        try!(project_conf.save(&path));
        path.pop();

        let payload = if path.is_relative() {
            try!(inapi::Payload::new(&dirname))
        } else {
            try!(inapi::Payload::new(path.to_str().unwrap()))
        };

        Ok(payload)
    }
}

#[derive(Debug)]
pub enum PayloadError {
    CreateFailed(String),
    InvalidPath,
    PayloadExists,
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PayloadError::CreateFailed(ref e) => write!(f, "Could not create payload: {}", e),
            PayloadError::InvalidPath => write!(f, "Invalid path to payload"),
            PayloadError::PayloadExists => write!(f, "Payload already exists"),
        }
    }
}

impl error::Error for PayloadError {
    fn description(&self) -> &str {
        match *self {
            PayloadError::CreateFailed(_) => "Could not create payload",
            PayloadError::InvalidPath => "Invalid path to payload",
            PayloadError::PayloadExists => "Payload already exists",
        }
    }
}

#[cfg(test)]
mod tests {
    use language::Language;
    use project::Project;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_find() {
        let dir = TempDir::new("test_rust_init").unwrap();
        let mut path = dir.path().to_owned();

        path.push("proj");
        Project::create(&path, Language::Rust).unwrap();

        path.push("payloads/payload1");
        Payload::create(&path, Language::Rust).unwrap();
        path.pop();

        path.push("payload2");
        Payload::create(&path, Language::Rust).unwrap();
        path.pop();

        path.push("payload3");
        Payload::create(&path, Language::Rust).unwrap();
        path.pop();

        let payloads = Payload::find(&path, None).unwrap();
        assert_eq!(payloads.len(), 3);

        let payloads = Payload::find(&path, Some(&vec!["payload2".into(), "payload3".into()])).unwrap();
        assert_eq!(payloads.len(), 2);
    }
}
