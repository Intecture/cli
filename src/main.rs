// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate czmq;
extern crate docopt;
extern crate rustc_serialize;
#[cfg(test)]
extern crate tempdir;
extern crate time;
extern crate zdaemon;

mod auth;
mod cert;
mod config;
mod error;
mod language;
mod project;

use auth::Auth;
use docopt::Docopt;
use error::Result;
use project::Project;
use std::{env, io};
use std::error::Error;
use std::path::Path;
use std::process::exit;

const VERSION: &'static str = "0.1.0";

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli run [<arg>...]
  incli init [(-e | --example)] (<name> <lang>)
  incli host (add | delete | bootstrap) <hostname>
  incli host list
  incli user (add | delete) <username>
  incli user list
  incli (-h | --help)
  incli --version

Options:
  -e --example  Clone an example project.
  -h --help     Show this screen.
  -v --verbose  Verbose output.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_add: bool,
    cmd_bootstrap: bool,
    cmd_delete: bool,
    cmd_host: bool,
    cmd_init: bool,
    cmd_list: bool,
    cmd_run: bool,
    cmd_user: bool,
    flag_e: bool,
    flag_example: bool,
    flag_h: bool,
    flag_help: bool,
    flag_v: bool,
    flag_verbose: bool,
    flag_version: bool,
    arg_arg: Vec<String>,
    arg_hostname: String,
    arg_lang: String,
    arg_name: String,
    arg_username: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}", VERSION);
    }
    else if args.cmd_run {
        let project = try_exit(Project::load(&mut env::current_dir().unwrap()), args.flag_v || args.flag_verbose);
        try_exit(project.run(&args.arg_arg), args.flag_v || args.flag_verbose);
    }
    else if args.cmd_init {
        try_exit(Project::create(&Path::new(&args.arg_name), &args.arg_lang, args.flag_e || args.flag_example), args.flag_v || args.flag_verbose);
    }
    else if args.cmd_host || args.cmd_user {
        let cert_type = if args.cmd_host { "host" } else { "user" };
        let name = if args.cmd_host { &args.arg_hostname } else { &args.arg_username };

        let auth = try_exit(Auth::new(&env::current_dir().unwrap()), args.flag_v || args.flag_verbose);

        if args.cmd_add {
            let cert = try_exit(auth.add(cert_type, name), args.flag_v || args.flag_verbose);
            println!("Please distribute this certificate securely.

------------------------COPY BELOW THIS LINE-------------------------
{}
------------------------COPY ABOVE THIS LINE-------------------------", cert.public());
        }
        else if args.cmd_delete {
            println!("Are you sure you want to delete this certificate?");
            loop {
                println!("Please enter [y/n]: ");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => match { input.as_ref() as &str }.trim() {
                        "y" => {
                            try_exit(auth.delete(name), args.flag_v || args.flag_verbose);
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
        else if args.cmd_list {
            let names = try_exit(auth.list(cert_type), args.flag_v || args.flag_verbose);

            for name in names {
                println!("{}", name);
            }
        }
        else if args.cmd_bootstrap && args.cmd_host {
            unimplemented!();
        }
    }
}

fn try_exit<T>(r: Result<T>, verbose: bool) -> T {
    if let Err(e) = r {
        println!("{}", e);

        if verbose {
            println!("{:?}", e.cause());
        }

        exit(1);
    }

    r.unwrap()
}
