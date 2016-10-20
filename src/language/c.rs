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
use super::{LanguageProject, LanguageError};

const MAKEFILE_SOURCE: &'static [u8] = b"P=bootstrap
CFLAGS = -g -Wall
LDLIBS = -I /usr/local/include -L /usr/local/lib -linapi
CC = gcc

$(P): src/main.c
\t$(CC) $(CFLAGS) -o $@ $< $(LDLIBS)

clean:
\trm $(P)
";

const BOOTSTRAP_SOURCE: &'static [u8] = b"#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inapi.h>

int main (int argc, char *argv[]) {
    if (argc < 2) {
        printf(\"Usage: incli run <server_host_or_ip>\\n\");
        return 1;
    }

    printf(\"Connecting to host %s...\", argv[1]);
    char *host_file = malloc(256 * sizeof(char));
    snprintf(host_file, sizeof host_file, \"hosts/%s.json\", argv[1]);
    Host *host = host_connect(host_file);
    if (host) {
        printf(\"done\\n\");
    } else {
        printf(\"\\nCouldn't connect to %s: %s\\n\", argv[1], geterr());
        return 1;
    }

    // Call payloads
    enum DataType payloads_dt = Array;
    ValueArray *payloads = get_value(host->data, payloads_dt, \"/_payloads\");

    if (payloads) {
        int i;
        for (i = 0; i < payloads->length; i++) {
            enum DataType payload_dt = String;
            char *payload_name = get_value(payloads->ptr[i], payload_dt, NULL);
            assert(payload_name);
            printf(\"Running payload %s...\", payload_name);
            Payload *payload = payload_new(payload_name);
            if (!payload) {
                printf(\"\\nCouldn't create payload: %s\\n\", geterr());
                return 1;
            }

            int rc = payload_run(payload, host, NULL, 0);
            if (rc != 0) {
                printf(\"\\nCouldn't run payload: %s\\n\", geterr());
                return 1;
            }

            printf(\"ok\\n\");
        }
    }
}
";

pub struct CProject;

impl LanguageProject for CProject {
    fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        let mut buf = path.as_ref().to_owned();

        // Add bootstrap binary .gitignore
        buf.push(".gitignore");
        let mut fh = try!(fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&buf));
        try!(fh.write_all(b"bootstrap\n"));
        buf.pop();

        // Create Makefile
        buf.push("Makefile");
        let mut fh = try!(fs::File::create(&buf));
        try!(fh.write_all(MAKEFILE_SOURCE));
        buf.pop();

        buf.push("src");
        try!(fs::create_dir(&buf));

        // Write bootstrap source code to main.c
        buf.push("main.c");
        let mut fh = try!(fs::File::create(&buf));
        try!(fh.write_all(BOOTSTRAP_SOURCE));

        Ok(())
    }

    fn run(args: &[&str]) -> Result<ExitStatus> {
        // Attempt to build project before running it
        if fs::metadata("Makefile").is_ok() {
            let output = try!(Command::new("make").output());
            if !output.status.success() {
                return Err(LanguageError::BuildFailed(try!(String::from_utf8(output.stderr))).into());
            }
        }

        Ok(try!(Command::new("bootstrap")
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
        let dir = TempDir::new("test_c_init").unwrap();
        let mut path = dir.path().to_owned();

        path.push("proj");
        Project::create(&path, Language::C).unwrap();

        path.push("Makefile");
        assert!(path.exists());
        path.pop();

        path.push("src/main.c");
        assert!(path.exists());
    }
}
