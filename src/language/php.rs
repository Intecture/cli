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

const BOOTSTRAP_SOURCE: &'static [u8] = b"<?php

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
        echo \"Running payload $payload_name...\";
        $payload = new Payload($payload_name);
        $payload->run($host);
        echo 'ok', PHP_EOL;
    }
}
";

pub struct PhpProject;

impl LanguageProject for PhpProject {
    fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        let mut buf = path.as_ref().to_owned();

        buf.push("src");
        try!(fs::create_dir(&buf));

        buf.push("main.php");
        let mut fh = try!(fs::File::create(&buf));
        try!(fh.write_all(BOOTSTRAP_SOURCE));

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
    use project::Project;
    use tempdir::TempDir;

    #[test]
    fn test_init() {
        let dir = TempDir::new("test_php_init").unwrap();
        let mut path = dir.path().to_owned();

        path.push("proj");
        Project::create(&path, Language::Php).unwrap();

        path.push("src/main.php");
        assert!(path.exists());
    }
}
