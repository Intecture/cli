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

mod cert;
mod config;
mod error;
mod language;
mod host;
mod project;

use docopt::Docopt;
use host::Host;
use project::Project;
use std::{env, io, result};
use std::error::Error;
use std::path::Path;
use std::process::exit;

pub type Result<T> = result::Result<T, error::Error>;

const VERSION: &'static str = "0.1.0";

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli [(-v | --verbose)] run [<arg>...]
  incli [(-v | --verbose)] init [(-b | --blank)] (<name> <lang>)
  incli host (add | remove | bootstrap) <hostname>
  incli (-h | --help)
  incli --version

Options:
  -b --blank    Create a blank project.
  -h --help     Show this screen.
  -v --verbose  Verbose output.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_run: bool,
    cmd_init: bool,
    cmd_host: bool,
    cmd_add: bool,
    cmd_remove: bool,
    cmd_bootstrap: bool,
    flag_h: bool,
    flag_help: bool,
    flag_v: bool,
    flag_verbose: bool,
    flag_b: bool,
    flag_blank: bool,
    flag_version: bool,
    arg_arg: Vec<String>,
    arg_name: String,
    arg_lang: String,
    arg_hostname: String,
}

#[cfg_attr(test, allow(dead_code))]
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
        try_exit(Project::create(&Path::new(&args.arg_name), &args.arg_lang, args.flag_b || args.flag_blank), args.flag_v || args.flag_verbose);
    }
    else if args.cmd_host {
        let host = Host::new(&args.arg_hostname);

        if args.cmd_add {
            try_exit(host.add(), args.flag_v || args.flag_verbose);
        }
        else if args.cmd_remove {
            println!("Are you sure you want to delete this host certificate?");
            loop {
                println!("Please enter [y/n]: ");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => match { input.as_ref() as &str }.trim() {
                        "y" => {
                            try_exit(host.remove(), args.flag_v || args.flag_verbose);
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
        else if args.cmd_bootstrap {
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
