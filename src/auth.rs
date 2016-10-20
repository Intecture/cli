// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use cert::Cert;
use config::Config;
use czmq::{ZCert, ZFrame, ZMsg, ZSock, ZSockType};
use error::Result;
use std::{error, fmt};
use std::path::Path;
use zdaemon::ConfigFile;

pub struct Auth {
    sock: ZSock,
}

impl Auth {
    pub fn new<P: AsRef<Path>>(project_path: P) -> Result<Auth> {
        let mut buf = project_path.as_ref().to_owned();

        buf.push("project.json");
        let config = try!(Config::load(&buf));
        buf.pop();

        buf.push("auth.crt");
        let auth_cert = try!(ZCert::load(buf.to_str().unwrap()));
        buf.pop();

        buf.push("user.crt");
        let user_cert = try!(ZCert::load(buf.to_str().unwrap()));
        buf.pop();

        let sock = ZSock::new(ZSockType::REQ);
        user_cert.apply(&sock);
        sock.set_curve_serverkey(auth_cert.public_txt());
        sock.set_sndtimeo(Some(5000));
        sock.set_rcvtimeo(Some(5000));
        try!(sock.connect(&format!("tcp://{}", config.auth_server)));

        Ok(Auth {
            sock: sock,
        })
    }

    pub fn list(&self, cert_type: &str) -> Result<Vec<String>> {
        let req = ZMsg::new();
        try!(req.addstr("cert::list"));
        try!(req.addstr(cert_type));
        try!(req.send(&self.sock));

        let result = try!(ZFrame::recv(&self.sock));

        match try!(try!(result.data()).or(Err(Error::HostResponse))).as_ref() {
            "Ok" => {
                let reply = try!(ZMsg::recv(&self.sock));
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

    pub fn add(&self, cert_type: &str, name: &str) -> Result<Cert> {
        let req = ZMsg::new();
        try!(req.addstr("cert::create"));
        try!(req.addstr(cert_type));
        try!(req.addstr(name));
        try!(req.send(&self.sock));

        let result = try!(ZFrame::recv(&self.sock));

        match try!(try!(result.data()).or(Err(Error::HostResponse))).as_ref() {
            "Ok" => {
                let reply = try!(ZMsg::recv(&self.sock));

                if reply.size() != 3 {
                    return Err(Error::HostResponse.into())
                }

                let pubkey = try!(reply.popstr().unwrap().or(Err(Error::HostResponse)));
                let seckey = try!(reply.popstr().unwrap().or(Err(Error::HostResponse)));
                let meta = try!(reply.popbytes()).unwrap();

                let cert = Cert::new(ZCert::from_txt(&pubkey, &seckey));
                try!(cert.decode_meta(&meta));
                Ok(cert)
            },
            "Err" => {
                let e = try!(try!(self.sock.recv_str()).or(Err(Error::HostResponse)));
                Err(Error::HostError(e).into())
            },
            _ => Err(Error::HostResponse.into()),
        }
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        let req = ZMsg::new();
        try!(req.addstr("cert::delete"));
        try!(req.addstr(name));
        try!(req.send(&self.sock));

        let result = try!(ZFrame::recv(&self.sock));

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
    use config::Config;
    use czmq::{ZCert, ZMsg, ZSys};
    use language::Language;
    use std::thread::spawn;
    use super::*;
    use tempdir::TempDir;
    use zdaemon::ConfigFile;

    #[test]
    fn test_new() {
        let dir = TempDir::new("auth_test_new").unwrap();

        let mut path = dir.path().to_owned();

        path.push("project.json");
        let config = Config {
            language: Language::Rust,
            auth_server: "127.0.0.1:7101".into()
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

        let (client, server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::list");
            assert_eq!(&req.popstr().unwrap().unwrap(), "host");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("Fat").unwrap();
            rep.addstr("Yak").unwrap();
            rep.addstr("is").unwrap();
            rep.addstr("DELICIOUS").unwrap();
            rep.send(&server).unwrap();
        });

        let auth = Auth {
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

        let (client, server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::create");
            assert_eq!(&req.popstr().unwrap().unwrap(), "host");
            assert_eq!(&req.popstr().unwrap().unwrap(), "foobar");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0000000000000000000000000000000000000000").unwrap();
            rep.addstr("0000000000000000000000000000000000000000").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Err").unwrap();
            rep.addstr("I'm broke!").unwrap();
            rep.send(&server).unwrap();
        });

        let auth = Auth {
            sock: client,
        };
        assert!(auth.add("host", "foobar").is_ok());
        assert!(auth.add("host", "foobar").is_err());

        handle.join().unwrap();
    }

    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let handle = spawn(move|| {
            let req = ZMsg::recv(&server).unwrap();
            assert_eq!(&req.popstr().unwrap().unwrap(), "cert::delete");
            assert_eq!(&req.popstr().unwrap().unwrap(), "foobar");

            server.send_str("Ok").unwrap();
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Err").unwrap();
            rep.addstr("I'm broke!").unwrap();
            rep.send(&server).unwrap();
        });

        let auth = Auth {
            sock: client,
        };
        assert!(auth.delete("foobar").is_ok());
        assert!(auth.delete("foobar").is_err());

        handle.join().unwrap();
    }
}
