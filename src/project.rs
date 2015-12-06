// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::{Config, ConfigError};
use config::project::ProjectConf;
use language::Language;
use std::{fs, io};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub language: &'static Language,
    pub artifact: String,
}

impl <'a>Project {
    pub fn new(current_dir: &mut PathBuf) -> Result<Project, ProjectError<'a>> {
        match Self::get_config(current_dir) {
            Ok(conf) => {
                let project_name = current_dir.iter().last().unwrap().to_str().unwrap();

                let language: &Language;
                match Language::find(&conf.language) {
                    Some(l) => language = l,
                    None => return Err(ProjectError {
                        message: "Unsupported language",
                        root: RootError::None(()),
                    }),
                }

                let mut artifact = conf.artifact;

                // If we are calling a binary and the user hasn't
                // specified a path to that binary, prepend "./" so
                // that it can be invoked as a command.
                if language.runtime.is_none() && !&artifact.chars().any(|c: char| c == '/') {
                    artifact = format!("./{}", artifact);
                }

                Ok(Project {
                    name: project_name.to_string(),
                    language: language,
                    artifact: artifact,
                })
            },
            Err(e) => {
                Err(ProjectError {
                    message: "Could not load project.json",
                    root: RootError::ConfigError(e),
                })
            }
        }
    }

    fn get_config(path: &mut PathBuf) -> Result<ProjectConf, ConfigError> {
        path.push("project.json");
        let conf = try!(ProjectConf::load(&path));
        Ok(conf)
    }

    pub fn create(name: &str, lang_name: &str, is_blank: bool) -> Result<(), ProjectError<'a>> {
        // Check that language is valid
        let language: &Language;
        match Language::find(lang_name) {
            Some(l) => language = l,
            None => return Err(ProjectError {
                message: "Unsupported language",
                root: RootError::None(()),
            }),
        }

        let mut project_path = PathBuf::from(name);

        // Make sure folder doesn't already exist
        if fs::metadata(&project_path).is_ok() {
            return Err(ProjectError {
                message: "Project already exists",
                root: RootError::None(()),
            });
        }

        // Clone example project or create empty project folder
        if is_blank {
            if let Some(e) = fs::create_dir(&project_path).err() {
                return Err(ProjectError {
                    message: "Could not create project directory",
                    root: RootError::IoError(e),
                });
            }
        } else {
            Command::new("git")
                .arg("clone")
                .arg(language.example_repo)
                .arg(name)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .unwrap();
        }

        // Create project.json
        let project_conf = ProjectConf::new(lang_name, language.artifact);
        project_path.push("project.json");
        if let Some(e) = ProjectConf::save(&project_conf, &project_path).err() {
            return Err(ProjectError {
                message: "Could not create project.json",
                root: RootError::ConfigError(e),
            });
        }

        Ok(())
    }
    
    pub fn run(&self, args: &Vec<String>) -> Result<ExitStatus, ProjectError<'a>> {
        let mut cmd: Command;

        if self.language.runtime.is_some() {
            cmd = Command::new(self.language.runtime.unwrap());
            cmd.arg(&self.artifact);
        } else {
            cmd = Command::new(&self.artifact);
        }

        // Stream command pipes to stdout and strerr
        let cmd_result = cmd.args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();

        match cmd_result {
            Ok(mut cmd_child) => {
                match cmd_child.wait() {
                    Ok(s) => Ok(s),
                    Err(e) => Err(ProjectError {
                        message: "Error while running project",
                        root: RootError::IoError(e),
                    })
                }
            }
            Err(e) => return Err(ProjectError {
                message: "Could not run project artifact",
                root: RootError::IoError(e),
            }),
        }
    }
}

#[derive(Debug)]
pub struct ProjectError<'a> {
    pub message: &'a str,
    pub root: RootError,
}

#[derive(Debug)]
pub enum RootError {
    None(()),
    ConfigError(ConfigError),
    IoError(io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::fs::{create_dir, File, metadata, remove_dir, remove_dir_all};
    use std::io::Write;

    #[test]
    fn test_new_noconf() {
        create_dir("test_new_noconf").unwrap();
        let mut path = PathBuf::from("test_new_noconf");

        let result = Project::new(&mut path);
        println!("{:?}", result);
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.message, "Could not load project.json");

        remove_dir_all("test_new_noconf").unwrap();
    }

    #[test]
    fn test_new_nolang() {
        create_dir("test_new_nolang").unwrap();
        let mut path = PathBuf::from("test_new_nolang");

        let test_path = Path::new("test_new_nolang/project.json");
        let mut file = File::create(&test_path).unwrap();
        file.write_all("{\"language\":\"NOLANG\",\"artifact\":\"none\"}".as_bytes()).unwrap();

        let result = Project::new(&mut path);
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.message, "Unsupported language");

        remove_dir_all("test_new_nolang").unwrap();
    }

    #[test]
    fn test_new_ok() {
        create_dir("test_new_ok").unwrap();
        let mut path = PathBuf::from("test_new_ok");

        let test_path = Path::new("test_new_ok/project.json");
        let mut file = File::create(&test_path).unwrap();
        file.write_all("{\"language\":\"php\",\"artifact\":\"index.php\"}".as_bytes()).unwrap();

        let result = Project::new(&mut path);
        assert!(result.is_ok());

        remove_dir_all("test_new_ok").unwrap();
    }

    #[test]
    fn test_create_nolang() {
        let result = Project::create("test_create_nolang", "NOLANG", true);
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.message, "Unsupported language");
    }

    #[test]
    fn test_create_exists() {
        create_dir("test_create_exists").unwrap();

        let result = Project::create("test_create_exists", "php", true);
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.message, "Project already exists");

        remove_dir("test_create_exists").unwrap();
    }

    #[test]
    fn test_create_ok() {
        let result = Project::create("test_create_ok", "php", true);
        assert!(result.is_ok());
        assert!(metadata("test_create_ok").is_ok());
        assert!(metadata("test_create_ok/project.json").is_ok());

        remove_dir_all("test_create_ok").unwrap();
    }
}