use config::Config;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct ProjectConf {
    pub language: String,
}

impl Config for ProjectConf {
    type ConfigFile = ProjectConf;
}

impl ProjectConf {
    pub fn new(lang: &str) -> ProjectConf {
        ProjectConf {
            language: lang.to_string()
        }
    }
}