// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate rustc_serialize;
extern crate docopt;

mod config;
mod project;
mod language;

use docopt::Docopt;
use project::{Project, ProjectError};
use std::process::exit;

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli [(-v | --verbose)] run [<arg>...]
  incli [(-v | --verbose)] init [(-b | --blank)] (<name> <lang>)
  incli (-h | --help)
  incli --version

Options:
  -b --blank	Create a blank project.
  -h --help     Show this screen.
  -v --verbose  Verbose output.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_run: bool,
    cmd_init: bool,
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
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let args: Args = Docopt::new(USAGE)
	    .and_then(|d| d.decode())
	    .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("0.0.1");
    } else if args.cmd_run == true {
        match Project::new() {
            Ok(project) => {
                let result = project.run(&args.arg_arg);

                if let Some(e) = result.err() {
                    println!("{:?}", e);
                    exit(1);
                }
            },
            Err(e) => print_err(&e, args.flag_v || args.flag_verbose),
        }
    } else if args.cmd_init == true {
        match Project::create(&args.arg_name, &args.arg_lang, args.flag_b || args.flag_blank) {
            Ok(_) => println!("Created project {}", args.arg_name),
            Err(e) => print_err(&e, args.flag_v || args.flag_verbose),
        }
    }
}

fn print_err(e: &ProjectError, verbose: bool) {
    println!("{}", e.message);
                
    if verbose {
        println!("{:?}", e.root);
    }
    
    exit(1);
}
