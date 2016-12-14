// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use auth::Auth;
use error::{Error, Result};
use inapi::ProjectConfig;
use project;
use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use zdaemon::ConfigFile;

const BOOTSTRAP_SOURCE: &'static str = "#!/bin/sh
set -u

main() {
    # Run any user-defined preinstall scripts
    {{PREINSTALL}}

    local _tmpdir=\"$(mktemp -d 2>/dev/null || ensure mktemp -d -t intecture)\"
    cd $_tmpdir

    # Install agent
    curl -sSf https://get.intecture.io | sh -s -- -y -d $_tmpdir agent || exit 1

    # Create agent cert
    cat << \"EOF\" > agent.crt
{{AGENTCERT}}
EOF

    # Create auth cert
    cat << \"EOF\" > auth.crt
{{AUTHCERT}}
EOF

    {{SUDO}} $_tmpdir/agent/installer.sh install_certs agent.crt auth.crt
    {{SUDO}} $_tmpdir/agent/installer.sh amend_conf auth_server \"{{AUTHHOST}}\"
    {{SUDO}} $_tmpdir/agent/installer.sh amend_conf auth_update_port {{AUTHPORT}}
    {{SUDO}} $_tmpdir/agent/installer.sh start_daemon

    # Check that inagent is up and running
    sleep 1
    local _pid=$(pidof inagent)
    if [ ! -n $_pid ]; then
        echo \"Failed to start inagent daemon\"
        exit 1
    fi

    # Run any user-defined postinstall scripts
    {{POSTINSTALL}}
}

main || exit 1
";

pub struct Bootstrap {
    hostname: String,
    _stream: TcpStream,
    session: Session,
    is_root: bool,
}

impl Bootstrap {
    pub fn new(hostname: &str,
               port: Option<u32>,
               username: Option<&str>,
               password: Option<&str>,
               identity_file: Option<&str>) -> Result<Bootstrap> {
        let tcp = TcpStream::connect(&*format!("{}:{}", hostname, port.unwrap_or(22)))?;
        let mut sess = Session::new().unwrap();
        sess.handshake(&tcp)?;

        let u = username.unwrap_or("root");
        if let Some(ref i) = identity_file {
            sess.userauth_pubkey_file(u, None, Path::new(i), None)?;
        }
        else if let Some(ref p) = password {
            sess.userauth_password(u, p).unwrap();
        } else {
            sess.userauth_agent(u)?;
        }

        if sess.authenticated() {
            Ok(Bootstrap {
                hostname: hostname.into(),
                _stream: tcp,
                session: sess,
                is_root: u == "root",
            })
        } else {
            Err(Error::Bootstrap("Failed to authenticate to host".into()))
        }
    }

    pub fn run(&mut self, preinstall_script: Option<&str>, postinstall_script: Option<&str>) -> Result<()> {
        let mut auth = try!(Auth::new(&env::current_dir().unwrap()));
        let agent_cert = try!(auth.add("host", &self.hostname));

        // As we are in a project directory, it's safe to assume that
        // the auth public key must be present.
        let mut fh = File::open("auth.crt")?;
        let mut auth_cert = String::new();
        fh.read_to_string(&mut auth_cert)?;

        // Load project config
        let conf = try!(ProjectConfig::load(project::CONFIGNAME));

        // Install and run bootstrap script
        let script = BOOTSTRAP_SOURCE.replace("{{AGENTCERT}}", &agent_cert.secret())
                                     .replace("{{AUTHCERT}}", &auth_cert)
                                     .replace("{{AUTHHOST}}", &conf.auth_server)
                                     .replace("{{AUTHPORT}}", &conf.auth_update_port.to_string())
                                     .replace("{{PREINSTALL}}", preinstall_script.unwrap_or(""))
                                     .replace("{{POSTINSTALL}}", postinstall_script.unwrap_or(""))
                                     .replace("{{SUDO}}", if self.is_root { "" } else { "sudo" });
        let bootstrap_path = self.channel_exec("mktemp 2>/dev/null || mktemp -t in-bootstrap")?;
        let cmd = format!("chmod u+x {0} && cat << \"EOS\" > {0}
{1}
EOS", bootstrap_path.trim(), script);
        self.channel_exec(&cmd)?;
        self.channel_exec(&bootstrap_path)?;

        Ok(())
    }

    fn channel_exec(&mut self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;
        let mut out = String::new();
        channel.read_to_string(&mut out)?;

        if channel.exit_status()? == 0 {
            Ok(out)
        } else {
            let mut stderr = channel.stderr();
            let mut err = String::new();
            stderr.read_to_string(&mut err)?;
            Err(Error::Bootstrap(format!("stdout: {}\nstderr: {}", out, err)))
        }
    }
}
