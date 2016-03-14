// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
use std::env;
use std::process::exit;

static USAGE: &'static str = "
Intecture CLI.

Usage:
  incli [(-v | --verbose)] run [<arg>...]
  incli [(-v | --verbose)] init [(-b | --blank)] (<name> <lang>)
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
        println!("0.0.2");
    } else if args.cmd_run == true {
        if let Some(e) = run(&args.arg_arg).err() {
            print_err(&e, args.flag_v || args.flag_verbose);
        }
    } else if args.cmd_init == true {
        if let Some(e) = init(&args.arg_name, &args.arg_lang, args.flag_b || args.flag_blank).err() {
            print_err(&e, args.flag_v || args.flag_verbose);
        }
    }
}

fn run<'a>(args: &Vec<String>) -> Result<(), ProjectError<'a>> {
    let mut current_dir = env::current_dir().unwrap();
    let project = try!(Project::new(&mut current_dir));
    try!(project.run(args));
    Ok(())
}

fn init<'a>(name: &str, lang_name: &str, is_blank: bool) -> Result<(), ProjectError<'a>> {
    Project::create(name, lang_name, is_blank)
}

fn print_err(e: &ProjectError, verbose: bool) {
    println!("{}", e.message);

    if verbose {
        println!("{:?}", e.root);
    }

    exit(1);
}
