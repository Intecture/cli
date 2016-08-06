// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use cert::Cert;
use config::Config;
use czmq::ZCert;
use error::{Error, Result};
use language::Language;
use std::{error, fmt};
use std::fs::{create_dir, File, metadata};
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use zdaemon::ConfigFile;

const EXAMPLE_REPO: &'static str = "https://github.com/intecture/examples";

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub language: &'static Language,
    pub artifact: String,
}

impl Project {
    pub fn load(project_path: &Path) -> Result<Project> {
        // Load config
        let mut buf = project_path.to_path_buf();
        buf.push("project.json");
        let conf = try!(Config::load(&buf));

        let project_name = project_path.iter().last().unwrap().to_str().unwrap();

        let language: &Language;
        match Language::find(&conf.language) {
            Some(l) => language = l,
            None => return Err(Error::from(ProjectError::InvalidLang)),
        }

        let mut artifact = conf.artifact;

        // If we are calling a binary and the user hasn't specified a
        // path to that binary, prepend "./" so that it can be
        // invoked as a command.
        if language.runtime.is_none() && !&artifact.chars().any(|c: char| c == '/') {
            artifact = format!("./{}", artifact);
        }

        Ok(Project {
            name: project_name.to_string(),
            language: language,
            artifact: artifact,
        })
    }

    pub fn create<P: AsRef<Path>>(project_path: P, lang_name: &str, is_example: bool) -> Result<()> {
        // Check that language is valid
        let language: &Language;
        match Language::find(lang_name) {
            Some(l) => language = l,
            None => return Err(Error::from(ProjectError::InvalidLang)),
        }

        // Make sure folder doesn't already exist
        if metadata(&project_path).is_ok() {
            return Err(Error::from(ProjectError::ProjectExists));
        }

        // Clone example project or create empty project folder
        if is_example {
            Command::new("git").args(&["clone", "-b", lang_name, EXAMPLE_REPO, project_path.as_ref().to_str().unwrap()])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output().unwrap();
        } else {
            try!(create_dir(&project_path));
        }

        // Create project.json
        let project_conf = Config::new(lang_name, language.artifact, "auth.example.com:7101");
        let mut buf = project_path.as_ref().to_path_buf();
        buf.push("project.json");
        try!(project_conf.save(&buf));

        // Create user certificate
        let zcert = try!(ZCert::new());
        let cert = Cert::new(zcert);
        let cert_path = format!("{}/user.crt", project_path.as_ref().to_str().unwrap());
        let mut cert_file = try!(File::create(&cert_path));
        try!(cert_file.write_all(cert.secret().as_bytes()));
        try!(Command::new("chmod").arg("600").arg(cert_path).status());

        println!("A new user certificate has been generated.

To complete the installation, please copy+paste this certificate into
the host's \"users\" directory.

------------------------COPY BELOW THIS LINE-------------------------
{}
------------------------COPY ABOVE THIS LINE-------------------------", cert.public());

        Ok(())
    }

    pub fn run(&self, args: &Vec<String>) -> Result<ExitStatus> {
        let mut cmd = match self.language.runtime {
            Some(runtime) => {
                let mut cmd = Command::new(runtime);
                cmd.arg(&self.artifact);
                cmd
            },
            None => Command::new(&self.artifact),
        };

        // Stream command pipes to stdout and strerr
        Ok(try!(cmd.args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()))
    }
}

#[derive(Debug)]
pub enum ProjectError {
    InvalidLang,
    ProjectExists,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProjectError::InvalidLang => write!(f, "Invalid language"),
            ProjectError::ProjectExists => write!(f, "Project already exists"),
        }
    }
}

impl error::Error for ProjectError {
    fn description(&self) -> &str {
        match *self {
            ProjectError::InvalidLang => "Invalid language",
            ProjectError::ProjectExists => "Project already exists",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, metadata};
    use std::io::Write;
    use std::path::Path;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_load_noconf() {
        let dir = TempDir::new("test_load_noconf").unwrap();
        assert!(Project::load(dir.path()).is_err());
    }

    #[test]
    fn test_load_nolang() {
        let dir = TempDir::new("test_load_nolang").unwrap();

        let mut buf = dir.path().to_path_buf();
        buf.push("project.json");

        let mut file = File::create(&buf).unwrap();
        file.write_all("{\"language\":\"NOLANG\",\"artifact\":\"none\"}".as_bytes()).unwrap();

        assert!(Project::load(&buf).is_err());
    }

    #[test]
    fn test_load_ok() {
        let dir = TempDir::new("test_load_ok").unwrap();
        let mut file = File::create(&format!("{}/project.json", dir.path().to_str().unwrap())).unwrap();
        file.write_all("{\"language\":\"php\",\"artifact\":\"index.php\",\"auth_server\":\"auth.example.com:7101\"}".as_bytes()).unwrap();
        assert!(Project::load(dir.path()).is_ok());
    }

    #[test]
    fn test_create_nolang() {
        assert!(Project::create(Path::new("/fake/path"), "NOLANG", false).is_err());
    }

    #[test]
    fn test_create_exists() {
        let dir = TempDir::new("test_create_exists").unwrap();
        assert!(Project::create(dir.path(), "php", false).is_err());
    }

    #[test]
    fn test_create_ok() {
        let dir = TempDir::new("test_create_ok").unwrap();
        let mut buf = dir.path().to_path_buf();
        buf.push("proj_dir");

        assert!(Project::create(&buf, "php", true).is_ok());
        assert!(metadata(format!("{}/project.json", buf.to_str().unwrap())).is_ok());
    }
}
