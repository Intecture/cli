// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::Config;
use error::Result;
use language::{Language, LanguageProject, CProject, PhpProject, RustProject};
use std::{error, fmt, fs};
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus};
use zdaemon::ConfigFile;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub language: Language,
}

impl Project {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Project> {
        // Load config
        let mut buf = path.as_ref().to_owned();
        buf.push("project.json");
        let conf = try!(Config::load(&buf));

        Ok(Project {
            name: try!(try!(buf.file_name().ok_or(ProjectError::InvalidPath))
                               .to_str().ok_or(ProjectError::InvalidPath)).into(),
            language: conf.language,
        })
    }

    pub fn create<P: AsRef<Path>>(project_path: P, language: Language) -> Result<Project> {
        let mut path = project_path.as_ref().to_owned();

        // Make sure folder doesn't already exist
        if path.exists() {
            return Err(ProjectError::ProjectExists.into());
        }

        path.push("data/hosts");
        try!(fs::create_dir_all(&path));
        path.pop();
        path.pop();

        path.push("payloads");
        try!(fs::create_dir(&path));
        path.pop();

        // Create git repo
        let output = try!(Command::new("git").args(&["init", path.to_str().unwrap()]).output());
        if !output.status.success() {
            return Err(ProjectError::CreateFailed(try!(String::from_utf8(output.stderr))).into());
        }

        // Update .gitignore
        path.push(".gitignore");
        let mut fh = try!(fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path));
        try!(fh.write_all(b"user.crt\nauth.crt\n"));
        path.pop();

        match language {
            Language::C => try!(CProject::init(&path)),
            Language::Php => try!(PhpProject::init(&path)),
            Language::Rust => try!(RustProject::init(&path)),
        }

        // Create project.json
        path.push("project.json");
        let project_conf = Config {
            language: language,
            auth_server: "auth.example.com:7101".into(),
        };
        try!(project_conf.save(&path));
        path.pop();

        println!("Remember to copy your user certificate to {}/user.crt.
If you do not have a user certificate, obtain one from your administrator.", path.to_str().unwrap());

        Ok(Project {
            name: try!(try!(path.file_name().ok_or(ProjectError::InvalidPath))
                                .to_str().ok_or(ProjectError::InvalidPath)).into(),
            language: project_conf.language,
        })
    }

    pub fn run(&self, args: &[&str]) -> Result<ExitStatus> {
        match self.language {
            Language::Php => PhpProject::run(args),
            Language::C => CProject::run(args),
            Language::Rust => RustProject::run(args),
        }
    }
}

#[derive(Debug)]
pub enum ProjectError {
    CreateFailed(String),
    InvalidPath,
    ProjectExists,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProjectError::CreateFailed(ref e) => write!(f, "Could not create project: {}", e),
            ProjectError::InvalidPath => write!(f, "Invalid path to project"),
            ProjectError::ProjectExists => write!(f, "Project already exists"),
        }
    }
}

impl error::Error for ProjectError {
    fn description(&self) -> &str {
        match *self {
            ProjectError::CreateFailed(_) => "Could not create project",
            ProjectError::InvalidPath => "Invalid path to project",
            ProjectError::ProjectExists => "Project already exists",
        }
    }
}

#[cfg(test)]
mod tests {
    use language::Language;
    use std::fs::{File, metadata};
    use std::io::Write;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_load_noconf() {
        let dir = TempDir::new("test_load_noconf").unwrap();
        assert!(Project::load(dir.path()).is_err());
    }

    #[test]
    fn test_load_ok() {
        let dir = TempDir::new("test_load_ok").unwrap();
        let mut path = dir.path().to_owned();

        path.push("project.json");
        let mut file = File::create(&path).unwrap();
        file.write_all("{\"language\":\"Php\",\"auth_server\":\"auth.example.com:7101\"}".as_bytes()).unwrap();
        path.pop();

        assert!(Project::load(&path).is_ok());
    }

    #[test]
    fn test_create_exists() {
        let dir = TempDir::new("test_create_exists").unwrap();
        assert!(Project::create(dir.path(), Language::Php).is_err());
    }

    #[test]
    fn test_create_ok() {
        let dir = TempDir::new("test_create_ok").unwrap();
        let mut buf = dir.path().to_owned();
        buf.push("proj_dir");

        assert!(Project::create(&buf, Language::Rust).is_ok());
        assert!(metadata(format!("{}/project.json", buf.to_str().unwrap())).is_ok());
    }
}
