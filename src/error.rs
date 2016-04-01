// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::Error as CError;
use host::HostError;
use project::ProjectError;
use rustc_serialize::json::{DecoderError, EncoderError};
use std::{error, fmt, io};
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Czmq(CError),
    Decoder(DecoderError),
    Encoder(EncoderError),
    Io(io::Error),
    Host(HostError),
    Project(ProjectError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Czmq(ref e) => write!(f, "CZMQ error: {}", e),
            Error::Decoder(ref e) => write!(f, "Decoder error: {}", e),
            Error::Encoder(ref e) => write!(f, "Encoder error: {}", e),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::Host(ref e) => write!(f, "Host error: {}", e),
            Error::Project(ref e) => write!(f, "Project error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Czmq(ref e) => e.description(),
            Error::Decoder(ref e) => e.description(),
            Error::Encoder(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Host(ref e) => e.description(),
            Error::Project(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Czmq(ref e) => Some(e),
            Error::Decoder(ref e) => Some(e),
            Error::Encoder(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::Host(ref e) => Some(e),
            Error::Project(ref e) => Some(e),
        }
    }
}

impl From<CError> for Error {
    fn from(err: CError) -> Error {
        Error::Czmq(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Error {
        Error::Decoder(err)
    }
}

impl From<EncoderError> for Error {
    fn from(err: EncoderError) -> Error {
        Error::Encoder(err)
    }
}

impl From<HostError> for Error {
    fn from(err: HostError) -> Error {
        Error::Host(err)
    }
}

impl From<ProjectError> for Error {
    fn from(err: ProjectError) -> Error {
        Error::Project(err)
    }
}
