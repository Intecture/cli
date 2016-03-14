// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

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