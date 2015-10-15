// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

pub mod project;

use rustc_serialize::{Decodable, Encodable, json};
use std::{io, convert, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub trait Config {
    type ConfigFile: Decodable + Encodable;

	fn load(file_path: &Path) -> Result<Self::ConfigFile, ConfigError> {
		let mut config_file = try!(fs::File::open(&file_path));
		let mut config_content = String::new();
		try!(config_file.read_to_string(&mut config_content));

		let config: Self::ConfigFile = json::decode(&config_content).unwrap();
		Ok(config)
	}

	fn save(config: &Self::ConfigFile, file_path: &Path) -> Result<(), ConfigError> {
        let mut file = try!(File::create(file_path));
        let json = json::encode(config).unwrap();

        try!(file.write_all(&json.as_bytes()));
        Ok(())
	}
}

#[derive(Debug)]
pub enum ConfigError {
	IoError(io::Error),
}

impl convert::From<io::Error> for ConfigError {
	fn from(err: io::Error) -> ConfigError {
		ConfigError::IoError(err)
	}
}