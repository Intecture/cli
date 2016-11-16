// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
#[cfg(test)]
extern crate tempdir;
extern crate time;
extern crate zdaemon;

mod auth;
mod cert;
mod error;
mod language;
mod payload;
mod project;

use auth::Auth;
use docopt::Docopt;
use error::Result;
use language::language_from_str;
use payload::Payload;
use project::Project;
use std::{env, io};
use std::path::Path;
use std::process::exit;

const API_VERSION: &'static str = "{ git = \"https://github.com/intecture/api\" }";
const VERSION: &'static str = "0.2.1";

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli run [<arg>...]
  incli project init <name> <lang>
  incli payload init <name> <lang>
  incli payload build [<names>...]
  incli host (add | delete | bootstrap) [(-s | --silent)] <hostname>
  incli host list
  incli user (add | delete) [(-s | --silent)] <username>
  incli user list
  incli (-h | --help)
  incli --version

Options:
  -h --help     Show this screen.
  -s --silent   Save private key instead of printing it.
  -v --verbose  Verbose output.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
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
    flag_s: bool,
    flag_silent: bool,
    flag_version: bool,
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

            for name in names {
                println!("{}", name);
            }
        }
        else if args.cmd_bootstrap && args.cmd_host {
            unimplemented!();
        }
    }

    Ok(())
}
