// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZCert, ZFrame, ZMsg, ZSock, SocketType};
use error::Result;
use inapi::ProjectConfig;
use language::{Language, LanguageProject, CProject, PhpProject, RustProject};
use {read_conf, write_conf};
use std::{error, fmt, fs};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CONFIGNAME: &'static str = "project.json";

#[derive(Debug)]
pub struct Project {
    path: PathBuf,
    pub name: String,
    conf: ProjectConfig,
}

impl Project {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Project> {
        let mut buf = path.as_ref().to_owned();

        // Load config
        buf.push(CONFIGNAME);
        let conf: ProjectConfig = read_conf(&buf)?;
        buf.pop();

        let name = buf.file_name()
                      .ok_or(ProjectError::InvalidPath)?
                      .to_str()
                      .ok_or(ProjectError::InvalidPath)?
                      .into();

        Ok(Project {
            path: buf,
            name: name,
            conf: conf,
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
            Language::C => try!(CProject::init_project(&path)),
            Language::Php => try!(PhpProject::init_project(&path)),
            Language::Rust => try!(RustProject::init_project(&path)),
        }

        // Create project.json
        path.push(CONFIGNAME);
        let project_conf = ProjectConfig {
            language: language,
            auth_server: "auth.example.com".into(),
            auth_api_port: 7101,
            auth_update_port: 7102,
            build_server: None,
        };
        write_conf(&project_conf, &path)?;
        path.pop();

        println!("Remember to copy your user certificate to {}/user.crt.
If you do not have a user certificate, obtain one from your administrator.", path.to_str().unwrap());

        let name = path.file_name()
                       .ok_or(ProjectError::InvalidPath)?
                       .to_str()
                       .ok_or(ProjectError::InvalidPath)?
                       .into();

        Ok(Project {
            path: path,
            name: name,
            conf: project_conf,
        })
    }

    pub fn run(&self, args: &[&str], local: bool) -> Result<Option<i32>> {
        match self.conf.build_server {
            Some(ref hostname) if !local => {
                let mut buf = self.path.clone();

                buf.push("build.crt");
                let build_cert = try!(ZCert::load(buf.to_str().unwrap()));
                buf.pop();

                buf.push("user.crt");
                let user_cert = try!(ZCert::load(buf.to_str().unwrap()));
                buf.pop();

                let mut sock = ZSock::new(SocketType::DEALER);
                user_cert.apply(&mut sock);
                sock.set_curve_serverkey(build_cert.public_txt());
                sock.set_sndtimeo(Some(1000));
                try!(sock.connect(&format!("tcp://{}", hostname)));

                let msg = ZMsg::new();
                msg.addstr("RUN")?;
                for arg in args {
                    msg.addstr(arg)?;
                }
                msg.send(&mut sock)?;

                let mut err = false;

                loop {
                    let reply = ZFrame::recv(&mut sock)?;
                    match reply.data()? {
                        Ok(ref data) if data == "OK" => break,
                        Ok(ref data) if data == "ERR" => err = true,
                        Ok(data) => {
                            println!("{}", data.trim());
                            if err {
                                break;
                            }
                        },
                        Err(bytes) => println!("{}", String::from_utf8_lossy(&bytes).trim()),
                    }
                }

                if err {
                    Ok(Some(1))
                } else {
                    Ok(Some(0))
                }
            }
            _ => {
                let status = match self.conf.language {
                    Language::Php => PhpProject::run(args)?,
                    Language::C => CProject::run(args)?,
                    Language::Rust => RustProject::run(args)?,
                };
                Ok(status.code())
            }
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

        path.push(CONFIGNAME);
        let mut file = File::create(&path).unwrap();
        file.write_all("{
            \"language\":\"Php\",
            \"auth_server\":\"auth.example.com\",
            \"auth_api_port\": 7101,
            \"auth_update_port\": 7102
        }".as_bytes()).unwrap();
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
        assert!(metadata(format!("{}/{}", buf.to_str().unwrap(), CONFIGNAME)).is_ok());
    }
}
