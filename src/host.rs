// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use cert::Cert;
use czmq::ZCert;
use error::Error;
use Result;
use std::{error, fmt};
use std::fs::{File, metadata, remove_file};
use std::io::Write;

pub const HOSTS_DIR: &'static str = ".hosts";

pub struct Host<'a> {
    hostname: &'a str,
    path: String,
}

impl<'a> Host<'a> {
    pub fn new(hostname: &str) -> Host {
        Host {
            hostname: hostname,
            path: format!("{}/{}.crt", HOSTS_DIR, hostname),
        }
    }

    pub fn exists(&self) -> bool {
        let md = metadata(&self.path);
        md.is_ok() && md.unwrap().is_file()
    }

    pub fn add(&self) -> Result<()> {
        if self.exists() {
            return Err(Error::from(HostError::AddExisting));
        }

        let zcert = try!(ZCert::new());
        let cert = Cert::new(&zcert);
        let mut cert_file = try!(File::create(&self.path));
        try!(cert_file.write_all(cert.public().as_bytes()));

        println!("A new host certificate has been generated for {}.

To complete the installation, please copy+paste this certificate into
the host's certificate path.

------------------------COPY BELOW THIS LINE-------------------------
{}
------------------------COPY ABOVE THIS LINE-------------------------", self.hostname, cert.secret());

        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        if !self.exists() {
            return Err(Error::from(HostError::RemoveNonexistent));
        }

        try!(remove_file(&self.path));
        Ok(())
    }
}

#[derive(Debug)]
pub enum HostError {
    AddExisting,
    RemoveNonexistent,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HostError::AddExisting => write!(f, "Cannot add existing host"),
            HostError::RemoveNonexistent => write!(f, "Cannot remove non-existent host")
        }
    }
}

impl error::Error for HostError {
    fn description(&self) -> &str {
        match *self {
            HostError::AddExisting => "Cannot add existing host",
            HostError::RemoveNonexistent => "Cannot remove non-existent host",
        }
    }
}
