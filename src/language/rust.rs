// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use API_VERSION;
use error::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use super::{LanguageProject, LanguageError};

const BOOTSTRAP_SOURCE: &'static [u8] = b"#[macro_use]
extern crate inapi;

use inapi::{Error, Host, Payload};
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!(\"Usage: incli run <server_host_or_ip>\");
        exit(1);
    }

    if let Err(e) = run(&args[1]) {
        println!(\"\"); // Output line break
        println!(\"{}\", e);
        exit(1);
    }
}

fn run(name: &str) -> Result<(), Error> {
    print!(\"Connecting to host {}...\", name);
    let mut host = try!(Host::connect(&format!(\"hosts/{}.json\", name)));
    println!(\"done\");

    // Call payloads
    let data = host.data_owned();
    for name in try!(needarray!(data => \"/_payloads\")) {
        print!(\"Running payload nginx...\");
        let payload = try!(Payload::new(try!(needstr!(name))));
        try!(payload.run(&mut host, None));
        println!(\"ok\");
    }

    Ok(())
}
";

pub struct RustProject;

impl LanguageProject for RustProject {
    fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        let mut buf = path.as_ref().to_owned();

        // Init new Cargo project
        let output = try!(Command::new("cargo").args(&[
            "init",
            "--bin".into(),
            buf.to_str().unwrap(),
        ]).output());

        if !output.status.success() {
            return Err(LanguageError::CreateFailed(try!(String::from_utf8(output.stderr))).into());
        }

        // Add intecture-api as project dependency
        buf.push("Cargo.toml");
        let mut fh = try!(fs::OpenOptions::new().append(true).open(&buf));
        try!(fh.write_all(format!("intecture-api = {}", API_VERSION).as_bytes()));
        buf.pop();

        // Write bootstrap source code to main.rs
        buf.push("src/main.rs");
        let mut fh = try!(fs::OpenOptions::new().write(true).open(&buf));
        try!(fh.write_all(BOOTSTRAP_SOURCE));

        Ok(())
    }

    fn run(args: &[&str]) -> Result<ExitStatus> {
        Ok(try!(Command::new("cargo")
                        .args(&["run", "--release", "--"])
                        .args(args)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()))
    }
}

#[cfg(test)]
mod tests {
    use language::Language;
    use project::Project;
    use tempdir::TempDir;

    #[test]
    fn test_init() {
        let dir = TempDir::new("test_rust_init").unwrap();
        let mut path = dir.path().to_owned();

        path.push("proj");
        Project::create(&path, Language::Rust).unwrap();

        path.push("Cargo.toml");
        assert!(path.exists());
        path.pop();

        path.push("src/main.rs");
        assert!(path.exists());
    }
}
