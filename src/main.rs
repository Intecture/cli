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
  incli [(-v | --verbose)] init (<name> <lang>)
  incli (-h | --help)
  incli --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  -v --verbose  Verbose output.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_run: bool,
    cmd_init: bool,
    flag_h: bool,
    flag_help: bool,
	flag_v: bool,
    flag_verbose: bool,
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

    if args.cmd_run == true {
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
        match Project::create(&args.arg_name, &args.arg_lang) {
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
