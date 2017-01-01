// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use API_VERSION;
use error::Result;
use project::ProjectError;
use std::{env, fs};
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use super::{LanguageProject, LanguageError};

const PAYLOAD_SOURCE: &'static [u8] = b"#[macro_use]
extern crate inapi;

use inapi::*;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!(\"Missing Host endpoints\");
        exit(1);
    }

    if let Err(e) = run(&args[1], &args[2]) {
        println!(\"\"); // Output line break
        println!(\"{}\", e);
        exit(1);
    }
}

fn run(api_endpoint: &str, file_endpoint: &str) -> Result<(), Error> {
    let mut host = try!(Host::connect_payload(api_endpoint, file_endpoint));

    // Do stuff...

    Ok(())
}
";

const PROJECT_SOURCE: &'static [u8] = b"#[macro_use]
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
        println!(\"Running payload {}...\", name);
        let payload = try!(Payload::new(try!(needstr!(name))));
        try!(payload.run(&mut host, None));
    }

    Ok(())
}
";

pub struct RustProject;

impl RustProject {
    fn init<P: AsRef<Path>>(path: P, source: &[u8]) -> Result<()> {
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

        // Write source code to main.rs
        buf.push("src/main.rs");
        let mut fh = try!(fs::OpenOptions::new().write(true).open(&buf));
        try!(fh.write_all(source));

        Ok(())
    }
}

impl LanguageProject for RustProject {
    fn init_payload<P: AsRef<Path>>(path: P) -> Result<()> {
        try!(RustProject::init(path, PAYLOAD_SOURCE));
        Ok(())
    }

    fn init_project<P: AsRef<Path>>(path: P) -> Result<()> {
        try!(RustProject::init(path, PROJECT_SOURCE));
        Ok(())
    }

    fn run(args: &[&str]) -> Result<ExitStatus> {
        let curdir = try!(env::current_dir());
        let dirname = try!(try!(curdir.file_stem().ok_or(ProjectError::InvalidPath))
                                       .to_str().ok_or(ProjectError::InvalidPath));

        let status = try!(Command::new("cargo")
                                  .args(&["build", "--release"])
                                  .stdout(Stdio::inherit())
                                  .stderr(Stdio::inherit())
                                  .status());

        if !status.success() {
            return Ok(status);
        }

        Ok(try!(Command::new(&format!("target/release/{}", dirname))
                        .args(args)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()))
    }
}

#[cfg(test)]
mod tests {
    use language::Language;
    use payload::Payload;
    use project::Project;
    use std::{fs, str};
    use std::io::Read;
    use super::{PAYLOAD_SOURCE, PROJECT_SOURCE};
    use tempdir::TempDir;

    #[test]
    fn test_init() {
        let dir = TempDir::new("test_rust_init").unwrap();
        let mut path = dir.path().to_owned();

        // Init project
        path.push("proj");
        Project::create(&path, Language::Rust).unwrap();

        path.push("Cargo.toml");
        assert!(path.exists());
        path.pop();

        path.push("src/main.rs");
        let mut fh = fs::File::open(&path).unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, str::from_utf8(PROJECT_SOURCE).unwrap());
        path.pop();
        path.pop();

        // Init payload
        path.push("payloads/nginx");
        Payload::create(&path, Language::Rust).unwrap();

        path.push("Cargo.toml");
        assert!(path.exists());
        path.pop();

        path.push("src/main.rs");
        let mut fh = fs::File::open(&path).unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, str::from_utf8(PAYLOAD_SOURCE).unwrap());
    }
}
