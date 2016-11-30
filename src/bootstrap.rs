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
use ssh2::{Channel, Session};
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

    # Install agent
    local _installdir=$(curl -sSf https://get.intecture.io | sh -- -k agent)

    if [ -z $_installdir]; then
        echo \"Install failed\"
        exit 1
    fi

    # Create agent cert
    local _agentcrt = mktemp
    cat <<EOF > $_agentcrt
{{AGENTCERT}}
EOF

    # Create auth cert
    local _authcrt = mktemp
    cat <<EOF > $_authcrt
{{AUTHCERT}}
EOF

    {{SUDO}} $_installdir/installer.sh install_certs $_agentcrt $_authcrt
    {{SUDO}} $_installdir/installer.sh amend_conf auth_server \"{{AUTHHOST}}\"
    {{SUDO}} $_installdir/installer.sh amend_conf auth_update_port {{AUTHPORT}}
    {{SUDO}} $_installdir/installer.sh start_daemon

    # Check that inagent is up and running
    if ! $(pgrep -q inagent); then
        echo \"Failed to start inagent daemon\"
        exit 1
    fi

    # Run any user-defined postinstall scripts
    {{POSTINSTALL}}
}

main
";

pub struct Bootstrap {
    hostname: String,
    session: Session,
    is_root: bool,
}

impl Bootstrap {
    pub fn new<P: AsRef<Path>>(hostname: &str,
                               port: Option<u32>,
                               username: Option<&str>,
                               password: Option<&str>,
                               identity_file: Option<P>) -> Result<Bootstrap> {
        let tcp = TcpStream::connect(&*format!("{}:{}", hostname, port.unwrap_or(22)))?;
        let mut sess = Session::new().unwrap();
        sess.handshake(&tcp)?;

        let u = username.unwrap_or("root");
        if let Some(ref i) = identity_file {
            sess.userauth_pubkey_file(u, None, i.as_ref(), None)?;
        }
        else if let Some(ref p) = password {
            sess.userauth_password(u, p).unwrap();
        } else {
            sess.userauth_agent(u)?;
        }

        Ok(Bootstrap {
            hostname: hostname.into(),
            session: sess,
            is_root: u == "root",
        })
    }

    pub fn run(&mut self, preinstall_script: Option<&str>, postinstall_script: Option<&str>) -> Result<()> {
        let mut channel = self.session.channel_session()?;

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
        let bootstrap_path = Self::channel_exec(&mut channel, "mktemp")?;
        let cmd = format!("chmod +x {0} && cat <<EOF > {0}
{1}
EOF", bootstrap_path, script);
        Self::channel_exec(&mut channel, &cmd)?;
        Self::channel_exec(&mut channel, &bootstrap_path)?;

        Ok(())
    }

    fn channel_exec(channel: &mut Channel, cmd: &str) -> Result<String> {
        channel.exec(cmd)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;

        if channel.exit_status()? == 0 {
            Ok(s)
        } else {
            Err(Error::Bootstrap(s))
        }
    }
}
