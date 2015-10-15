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
use std::{env, fs, io};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};

pub struct Project {
    pub name: String,
    pub language: &'static Language,
    pub artifact: String,
}

impl <'a>Project {
    pub fn new() -> Result<Project, ProjectError<'a>> {
        let mut current_dir = env::current_dir().unwrap();

        match Self::get_config(&mut current_dir) {
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

                Ok(Project {
                    name: project_name.to_string(),
                    language: language,
                    artifact: conf.artifact,
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
    
    pub fn run(&self, args: &Vec<String>) -> Result<ExitStatus, ProjectError> {
        let mut command: Child;
        let mut cmd: Command;

        if self.language.runtime.is_some() {
            cmd = Command::new(self.language.runtime.unwrap());
            cmd.arg(&self.artifact);
        } else {
            cmd = Command::new(&self.artifact);
        }

        let cmd_result = cmd.args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();

        match cmd_result {
            Ok(cmd) => command = cmd,
            Err(e) => return Err(ProjectError {
                message: "Could not run project",
                root: RootError::IoError(e),
            }),
        }

        match command.wait() {
    	    Ok(s) => Ok(s),
    	    Err(e) => Err(ProjectError {
	            message: "Error while running project",
	            root: RootError::IoError(e),
            })
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