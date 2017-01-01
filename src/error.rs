// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use auth;
use czmq;
use inapi;
use language::LanguageError;
use payload::PayloadError;
use project::ProjectError;
use rustc_serialize::json::{DecoderError, EncoderError};
use ssh2;
use std::{error, fmt, io, result, string};
use std::convert::From;
use zdaemon;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Auth(auth::Error),
    Bootstrap(String),
    Czmq(czmq::Error),
    Decoder(DecoderError),
    Encoder(EncoderError),
    Inapi(inapi::Error),
    Io(io::Error),
    Language(LanguageError),
    Payload(PayloadError),
    Project(ProjectError),
    Ssh2(ssh2::Error),
    StringConvert(string::FromUtf8Error),
    ZDaemon(zdaemon::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Auth(ref e) => write!(f, "Auth error: {}", e),
            Error::Bootstrap(ref e) => write!(f, "Bootstrap error: {}", e),
            Error::Czmq(ref e) => write!(f, "CZMQ error: {}", e),
            Error::Decoder(ref e) => write!(f, "Decoder error: {}", e),
            Error::Encoder(ref e) => write!(f, "Encoder error: {}", e),
            Error::Inapi(ref e) => write!(f, "Intecture API error: {}", e),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::Language(ref e) => write!(f, "Language error: {}", e),
            Error::Payload(ref e) => write!(f, "Payload error: {}", e),
            Error::Project(ref e) => write!(f, "Project error: {}", e),
            Error::Ssh2(ref e) => write!(f, "SSH2 error: {}", e),
            Error::StringConvert(ref e) => write!(f, "String conversion error: {}", e),
            Error::ZDaemon(ref e) => write!(f, "ZDaemon error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Auth(ref e) => e.description(),
            Error::Bootstrap(ref e) => e,
            Error::Czmq(ref e) => e.description(),
            Error::Decoder(ref e) => e.description(),
            Error::Encoder(ref e) => e.description(),
            Error::Inapi(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Language(ref e) => e.description(),
            Error::Payload(ref e) => e.description(),
            Error::Project(ref e) => e.description(),
            Error::Ssh2(ref e) => e.description(),
            Error::StringConvert(ref e) => e.description(),
            Error::ZDaemon(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Auth(ref e) => Some(e),
            Error::Bootstrap(_) => None,
            Error::Czmq(ref e) => Some(e),
            Error::Decoder(ref e) => Some(e),
            Error::Encoder(ref e) => Some(e),
            Error::Inapi(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::Language(ref e) => Some(e),
            Error::Payload(ref e) => Some(e),
            Error::Project(ref e) => Some(e),
            Error::Ssh2(ref e) => Some(e),
            Error::StringConvert(ref e) => Some(e),
            Error::ZDaemon(ref e) => Some(e),
        }
    }
}

impl From<auth::Error> for Error {
    fn from(err: auth::Error) -> Error {
        Error::Auth(err)
    }
}

impl From<czmq::Error> for Error {
    fn from(err: czmq::Error) -> Error {
        Error::Czmq(err)
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

impl From<inapi::Error> for Error {
    fn from(err: inapi::Error) -> Error {
        Error::Inapi(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<LanguageError> for Error {
    fn from(err: LanguageError) -> Error {
        Error::Language(err)
    }
}

impl From<PayloadError> for Error {
    fn from(err: PayloadError) -> Error {
        Error::Payload(err)
    }
}

impl From<ProjectError> for Error {
    fn from(err: ProjectError) -> Error {
        Error::Project(err)
    }
}

impl From<ssh2::Error> for Error {
    fn from(err: ssh2::Error) -> Error {
        Error::Ssh2(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::StringConvert(err)
    }
}

impl From<zdaemon::Error> for Error {
    fn from(err: zdaemon::Error) -> Error {
        Error::ZDaemon(err)
    }
}
