// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use cert::Cert;
use inapi::ProjectConfig;
use czmq::{ZCert, ZFrame, ZMsg, ZSock, SocketType};
use error::Result;
use project;
use std::{error, fmt};
use std::path::Path;
use zdaemon::{ConfigFile, ZMsgExtended};

pub struct Auth {
    sock: ZSock,
}

impl Auth {
    pub fn new<P: AsRef<Path>>(project_path: P) -> Result<Auth> {
        let mut buf = project_path.as_ref().to_owned();

        buf.push(project::CONFIGNAME);
        let config = try!(ProjectConfig::load(&buf));
        buf.pop();

        buf.push("auth.crt");
        let auth_cert = try!(ZCert::load(buf.to_str().unwrap()));
        buf.pop();

        buf.push("user.crt");
        let user_cert = try!(ZCert::load(buf.to_str().unwrap()));
        buf.pop();

        let mut sock = ZSock::new(SocketType::REQ);
        user_cert.apply(&mut sock);
        sock.set_curve_serverkey(auth_cert.public_txt());
        sock.set_sndtimeo(Some(5000));
        sock.set_rcvtimeo(Some(5000));
        try!(sock.connect(&format!("tcp://{}:{}", config.auth_server, config.auth_api_port)));

        Ok(Auth {
            sock: sock,
        })
    }

    pub fn list(&mut self, cert_type: &str) -> Result<Vec<String>> {
        let req = ZMsg::new();
        try!(req.addstr("cert::list"));
        try!(req.addstr(cert_type));
        try!(req.send(&mut self.sock));

        let result = try!(ZFrame::recv(&mut self.sock));

        match try!(try!(result.data()).or(Err(Error::HostResponse))).as_ref() {
            "Ok" => {
                let reply = try!(ZMsg::recv(&mut self.sock));
                let mut list = Vec::new();

                for frame in reply {
                    list.push(try!(try!(frame.data()).or(Err(Error::HostResponse))));
                }

                Ok(list)
            },
            "Err" => {
                let e = try!(try!(self.sock.recv_str()).or(Err(Error::HostResponse)));
                Err(Error::HostError(e).into())
            },
            _ => Err(Error::HostResponse.into()),
        }
    }

    pub fn add(&mut self, cert_type: &str, name: &str) -> Result<Cert> {
        let req = ZMsg::new();
        try!(req.addstr("cert::create"));
        try!(req.addstr(cert_type));
        try!(req.addstr(name));
        try!(req.send(&mut self.sock));

        let reply = ZMsg::expect_recv(&mut self.sock, 2, None, true)?;

        match reply.popstr().unwrap().or(Err(Error::HostResponse))?.as_ref() {
            "Ok" => {
                if reply.size() != 3 {
                    return Err(Error::HostResponse.into())
                }

                let pubkey = reply.popstr().unwrap().or(Err(Error::HostResponse))?;
                let seckey = reply.popstr().unwrap().or(Err(Error::HostResponse))?;
                let meta = reply.popbytes()?.unwrap();

                let cert = Cert::new(ZCert::from_txt(&pubkey, &seckey)?);
                try!(cert.decode_meta(&meta));
                Ok(cert)
            },
            "Err" => {
                let e = reply.popstr().unwrap().or(Err(Error::HostResponse))?;
                Err(Error::HostError(e).into())
            },
            _ => Err(Error::HostResponse.into()),
        }
    }

    pub fn delete(&mut self, name: &str) -> Result<()> {
        let req = ZMsg::new();
        try!(req.addstr("cert::delete"));
        try!(req.addstr(name));
        try!(req.send(&mut self.sock));

        let result = try!(ZFrame::recv(&mut self.sock));

        match try!(try!(result.data()).or(Err(Error::HostResponse))).as_ref() {
            "Ok" => Ok(()),
            "Err" => {
                let e = try!(try!(self.sock.recv_str()).or(Err(Error::HostResponse)));
                Err(Error::HostError(e).into())
            },
            _ => Err(Error::HostResponse.into()),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    HostError(String),
    HostResponse,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::HostError(ref e) => write!(f, "Auth server encountered an error: {}", e),
            Error::HostResponse => write!(f, "Invalid response from host"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::HostError(ref e) => e,
            Error::HostResponse => "Invalid response from host",
        }
    }
}

#[cfg(test)]
mod tests {
    use inapi::ProjectConfig;
    use czmq::{ZCert, ZMsg, ZSys};
    use language::Language;
    use project;
    use std::thread::spawn;
    use super::*;
    use tempdir::TempDir;
    use zdaemon::ConfigFile;

    #[test]
    fn test_new() {
        let dir = TempDir::new("auth_test_new").unwrap();

        let mut path = dir.path().to_owned();

        path.push(project::CONFIGNAME);
        let config = ProjectConfig {
            language: Language::Rust,
            auth_server: "127.0.0.1".into(),
            auth_api_port: 7101,
            auth_update_port: 0,
        };
        config.save(&path).unwrap();
        path.pop();

        path.push("auth.crt");
        let cert = ZCert::new().unwrap();
        cert.save_public(path.to_str().unwrap()).unwrap();
        path.pop();

        path.push("user.crt");
        let cert = ZCert::new().unwrap();
        cert.save_secret(path.to_str().unwrap()).unwrap();
        path.pop();

        Auth::new(&path).unwrap();
    }

    #[test]
    fn test_list() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::list");
            assert_eq!(&req.popstr().unwrap().unwrap(), "host");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("Fat").unwrap();
            rep.addstr("Yak").unwrap();
            rep.addstr("is").unwrap();
            rep.addstr("DELICIOUS").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut auth = Auth {
            sock: client,
        };

        let mut list = auth.list("host").unwrap();
        assert_eq!(list.pop().unwrap(), "DELICIOUS");
        assert_eq!(list.pop().unwrap(), "is");
        assert_eq!(list.pop().unwrap(), "Yak");
        assert_eq!(list.pop().unwrap(), "Fat");

        handle.join().unwrap();
    }

    #[test]
    fn test_add() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::create");
            assert_eq!(&req.popstr().unwrap().unwrap(), "host");
            assert_eq!(&req.popstr().unwrap().unwrap(), "foobar");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0000000000000000000000000000000000000000").unwrap();
            rep.addstr("0000000000000000000000000000000000000000").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Err").unwrap();
            rep.addstr("I'm broke!").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut auth = Auth {
            sock: client,
        };
        assert!(auth.add("host", "foobar").is_ok());
        assert!(auth.add("host", "foobar").is_err());

        handle.join().unwrap();
    }

    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::delete");
            assert_eq!(&req.popstr().unwrap().unwrap(), "foobar");

            server.send_str("Ok").unwrap();
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Err").unwrap();
            rep.addstr("I'm broke!").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut auth = Auth {
            sock: client,
        };
        assert!(auth.delete("foobar").is_ok());
        assert!(auth.delete("foobar").is_err());

        handle.join().unwrap();
    }
}
