use config::{Config, ConfigError};
use config::project::ProjectConf;
use language::Language;
use std::{env, fs, io};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};

pub struct Project {
    pub name: String,
    pub language: &'static Language,
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

    pub fn create(name: &str, lang_name: &str) -> Result<(), ProjectError<'a>> {
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

        // Create project folder
        if let Some(e) = fs::create_dir(&project_path).err() {
            return Err(ProjectError {
                message: "Could not create project directory",
                root: RootError::IoError(e),
            });
        }

        // Create project.json
        let project_conf = ProjectConf::new(lang_name);
        project_path.push("project.json");
        if let Some(e) = ProjectConf::save(&project_conf, &project_path).err() {
            return Err(ProjectError {
                message: "Could not create project.json",
                root: RootError::ConfigError(e),
            });
        }
        
        // Create bootstrap.<ext>
        project_path.set_file_name("bootstrap");
        project_path.set_extension(language.extension);
        
        let mut bootstrap_file: File;
        match File::create(&project_path) {
            Ok(f) => bootstrap_file = f,
            Err(e) => return Err(ProjectError {
                message: "Could not create bootstrap file",
                root: RootError::IoError(e),
            }),
        }
        
        if let Some(e) = bootstrap_file.write_all(language.bootstrap.as_bytes()).err() {
            return Err(ProjectError {
                message: "Could not write bootstrap file",
                root: RootError::IoError(e),
            });
        }

        Ok(())
    }
    
    pub fn run(&self, args: &Vec<String>) -> Result<ExitStatus, ProjectError> {
        let bootstrap_path = format!("bootstrap.{}", self.language.extension);
        let mut command: Child;
        let cmd_result = Command::new(self.language.runtime)
            .arg(bootstrap_path)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();
            
        match cmd_result {
            Ok(cmd) => command = cmd,
            Err(e) => return Err(ProjectError {
                message: "Could not run bootstrap",
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