// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate czmq;
extern crate docopt;
extern crate inapi;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate ssh2;
#[cfg(test)]
extern crate tempdir;
extern crate time;
extern crate zdaemon;

mod auth;
mod bootstrap;
mod cert;
mod error;
mod language;
mod payload;
mod project;

use auth::Auth;
use bootstrap::Bootstrap;
use docopt::Docopt;
use error::Result;
use language::language_from_str;
use payload::Payload;
use project::Project;
use serde::{Serialize, Deserialize};
use std::{env, fs};
use std::io::{Read, Write, self};
use std::path::Path;
use std::process::exit;

const API_VERSION: &'static str = "0.3";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli run [<arg>...]
  incli project init <name> <lang>
  incli payload init <name> <lang>
  incli payload build [<names>...]
  incli host (add | delete) [(-s | --silent)] <hostname>
  incli host bootstrap <hostname> [-u <username>] [-P <password>] [-i <identity_file>] [-p <ssh_port>] [-m <preinstall_script>] [-n <postinstall_script>]
  incli host list
  incli user (add | delete) [(-s | --silent)] <username>
  incli user list
  incli (-h | --help)
  incli --version

Options:
  -h --help                 Show this screen.
  -i <identity_file>        Path to SSH private key.
  -m <preinstall_script>    Script to run before attempting to install Agent.
  -n <postinstall_script>   Script to run after successfully installing Agent.
  -p <ssh_port>             SSH port number.
  -P <password>             SSH password.
  -s --silent               Save private key instead of printing it.
  -u <username>             SSH username.
  -v --verbose              Verbose output.
  --version                 Print this script's version.
";

#[derive(Debug, RustcDecodable)]
#[allow(non_snake_case)]
struct Args {
    cmd_add: bool,
    cmd_bootstrap: bool,
    cmd_build: bool,
    cmd_delete: bool,
    cmd_host: bool,
    cmd_init: bool,
    cmd_list: bool,
    cmd_payload: bool,
    cmd_project: bool,
    cmd_run: bool,
    cmd_user: bool,
    flag_h: bool,
    flag_help: bool,
    flag_i: Option<String>,
    flag_m: Option<String>,
    flag_n: Option<String>,
    flag_p: Option<u32>,
    flag_P: Option<String>,
    flag_s: bool,
    flag_silent: bool,
    flag_version: bool,
    flag_u: Option<String>,
    arg_arg: Vec<String>,
    arg_hostname: String,
    arg_lang: String,
    arg_name: String,
    arg_names: Option<Vec<String>>,
    arg_username: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if let Err(e) = run(&args) {
        println!("{}", e);
        println!("{:?}", e);
        exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    if args.flag_version {
        println!("{}", VERSION);
    }
    else if args.cmd_run {
        let project = try!(Project::load(&mut env::current_dir().unwrap()));
        let args_deref: Vec<&str> = args.arg_arg.iter().map(AsRef::as_ref).collect();
        let status = try!(project.run(&args_deref));

        if !status.success() {
            exit(status.code().unwrap_or(1));
        }
    }
    else if args.cmd_project && args.cmd_init {
        try!(Project::create(&Path::new(&args.arg_name), try!(language_from_str(&args.arg_lang))));
    }
    else if args.cmd_payload {
        if args.cmd_init {
            try!(Payload::create(&Path::new(&args.arg_name), try!(language_from_str(&args.arg_lang))));
        }
        else if args.cmd_build {
            let payloads = if let Some(ref names) = args.arg_names {
                let n: Vec<&str> = names.iter().map(|n| &**n).collect();
                try!(Payload::find(".", Some(&*n)))
            } else {
                try!(Payload::find(".", None))
            };
            for payload in payloads {
                try!(payload.build());
            }
        }
    }
    else if args.cmd_host || args.cmd_user {
        let cert_type = if args.cmd_host { "host" } else { "user" };
        let name = if args.cmd_host { &args.arg_hostname } else { &args.arg_username };

        let mut auth = try!(Auth::new(&env::current_dir().unwrap()));

        if args.cmd_add {
            let cert = try!(auth.add(cert_type, name));
            if args.flag_s || args.flag_silent {
                try!(cert.save_secret(&format!("{}.crt", name)));
            } else {
                println!("Please distribute this certificate securely.

------------------------COPY BELOW THIS LINE-------------------------
{}
------------------------COPY ABOVE THIS LINE-------------------------", cert.secret());
            }
        }
        else if args.cmd_delete {
            if args.flag_s || args.flag_silent {
                try!(auth.delete(name));
            } else {
                println!("Are you sure you want to delete this certificate?");
                loop {
                    println!("Please enter [y/n]: ");
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => match { input.as_ref() as &str }.trim() {
                            "y" => {
                                try!(auth.delete(name));
                                break;
                            },
                            "n" => break,
                            _ => (),
                        },
                        Err(e) => {
                            println!("Stdin error: {}", e);
                            exit(1);
                        },
                    }
                }
            }
        }
        else if args.cmd_list {
            let names = try!(auth.list(cert_type));

            if names.is_empty() {
                println!("No {}s found", cert_type);
            } else {
                for name in names {
                    println!("{}", name);
                }
            }
        }
        else if args.cmd_bootstrap && args.cmd_host {
            print!("Connecting to {}...", args.arg_hostname);
            let mut bootstrap = Bootstrap::new(&args.arg_hostname,
                                               args.flag_p,
                                               args.flag_u.as_ref().map(|u| &**u),
                                               args.flag_P.as_ref().map(|p| &**p),
                                               args.flag_i.as_ref().map(|i| &**i))?;
            println!("done");

            print!("Bootstrapping...");
            match bootstrap.run(args.flag_m.as_ref().map(|m| &**m), args.flag_n.as_ref().map(|n| &**n)) {
                Ok(()) => println!("done"),
                Err(e) => {
                    println!("error!");
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

fn read_conf<P: AsRef<Path>, T: Deserialize>(path: P) -> Result<T> {
    let mut fh = fs::File::open(path.as_ref())?;
    let mut json = String::new();
    fh.read_to_string(&mut json)?;
    Ok(serde_json::from_str(&json)?)
}

fn write_conf<P: AsRef<Path>, T: Serialize>(conf: T, path: P) -> Result<()> {
    let json = serde_json::to_string(&conf)?;
    let mut fh = fs::File::create(path.as_ref())?;
    fh.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use inapi::{Language, ProjectConfig};
    use project;
    use super::{read_conf, write_conf};
    use tempdir::TempDir;

    #[test]
    fn test_rw_conf() {
        let tmpdir = TempDir::new("test_rw_conf").unwrap();
        let mut path = tmpdir.path().to_owned();

        path.push(project::CONFIGNAME);
        let config = ProjectConfig {
            language: Language::Rust,
            auth_server: "127.0.0.1".into(),
            auth_api_port: 7101,
            auth_update_port: 0,
        };
        write_conf(&config, &path).unwrap();
        let _: ProjectConfig = read_conf(&path).unwrap();
    }
}
