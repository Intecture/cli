// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use super::LanguageProject;

const PAYLOAD_SOURCE: &'static [u8] = b"<?php

use Intecture\\Host;

if ($argc < 2) {
    echo 'Missing Host endpoints', PHP_EOL;
    exit(1);
}

$host = Host::connect_payload($argv[1], $argv[2]);

// Do stuff...
";

const PROJECT_SOURCE: &'static [u8] = b"<?php

use Intecture\\Host;
use Intecture\\Payload;

if ($argc < 2) {
    echo 'Usage: incli run <server_host_or_ip>', PHP_EOL;
    exit(1);
}

$hostname = $argv[1];

echo \"Connecting to host $hostname...\";
$host = Host::connect(\"hosts/$hostname.json\");
echo 'done', PHP_EOL;

// Call payloads
$data = $host->data();
if (array_key_exists('_payloads', $data)) {
    foreach ($data['_payloads'] as $payload_name) {
        echo \"Running payload $payload_name...\", PHP_EOL;
        $payload = new Payload($payload_name);
        $payload->run($host);
    }
}
";

pub struct PhpProject;

impl PhpProject {
    fn init<P: AsRef<Path>>(path: P, source: &[u8]) -> Result<()> {
        let mut buf = path.as_ref().to_owned();

        buf.push("src");
        try!(fs::create_dir(&buf));

        buf.push("main.php");
        let mut fh = try!(fs::File::create(&buf));
        try!(fh.write_all(source));

        Ok(())
    }
}

impl LanguageProject for PhpProject {
    fn init_payload<P: AsRef<Path>>(path: P) -> Result<()> {
        try!(PhpProject::init(path, PAYLOAD_SOURCE));
        Ok(())
    }

    fn init_project<P: AsRef<Path>>(path: P) -> Result<()> {
        try!(PhpProject::init(path, PROJECT_SOURCE));
        Ok(())
    }

    fn run(args: &[&str]) -> Result<ExitStatus> {
        Ok(try!(Command::new("php")
                        .arg("src/main.php")
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
        let dir = TempDir::new("test_php_init").unwrap();
        let mut path = dir.path().to_owned();

        // Init project
        path.push("proj");
        Project::create(&path, Language::Php).unwrap();

        path.push("src/main.php");
        let mut fh = fs::File::open(&path).unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, str::from_utf8(PROJECT_SOURCE).unwrap());
        path.pop();
        path.pop();

        // Init payload
        path.push("payloads/nginx");
        Payload::create(&path, Language::Php).unwrap();

        path.push("src/main.php");
        let mut fh = fs::File::open(&path).unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, str::from_utf8(PAYLOAD_SOURCE).unwrap());
    }
}
