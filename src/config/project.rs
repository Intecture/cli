use config::Config;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct ProjectConf {
    pub language: String,
    pub artifact: String,
}

impl Config for ProjectConf {
    type ConfigFile = ProjectConf;
}

impl ProjectConf {
    pub fn new(lang: &str, artifact: &str) -> ProjectConf {
        ProjectConf {
            language: lang.to_string(),
            artifact: artifact.to_string(),
        }
    }
}