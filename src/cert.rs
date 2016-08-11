// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::ZCert;
use std::ops::Deref;
use time;

pub struct Cert {
    zcert: ZCert,
}

impl Cert {
    pub fn new(zcert: ZCert) -> Cert {
        Cert {
            zcert: zcert,
        }
    }

    // pub fn public(&self) -> String {
    //     let mut c = self.header(false);
    //     self.add_metadata(&mut c);
    //
    //     c.push_str(&format!("curve
    // public-key = \"{}\"", self.zcert.public_txt()));
    //
    //     c
    // }

    pub fn secret(&self) -> String {
        let mut c = self.header(true);
        self.add_metadata(&mut c);

        c.push_str(&format!("curve
    public-key = \"{}\"
    secret-key = \"{}\"", self.zcert.public_txt(), self.zcert.secret_txt()));

        c
    }

    fn header(&self, secret: bool) -> String {
        let cert_type = if secret { "SECRET" } else { "Public" };
        let secret_warning = if secret {
            "#   THIS FILE MUST BE KEPT PRIVATE AT ALL TIMES AND NEVER SHARED!
"
        } else {
            ""
        };

        format!("#   ****  Generated on {} by incli  ****
#   Intecture CURVE {} Certificate
{}
", time::now().strftime("%F %T").unwrap(), cert_type, secret_warning)
    }

    // TODO: Need to handle metadata
    fn add_metadata(&self, header: &mut String) {
        header.push_str("metadata\n");
    }
}

impl Deref for Cert {
    type Target = ZCert;

    fn deref(&self) -> &ZCert {
        &self.zcert
    }
}

// These tests are pretty useless, but help code coverage
#[cfg(test)]
mod tests {
    use czmq::ZCert;
    use super::*;

    // #[test]
    // fn test_public() {
    //     let cert = Cert::new(ZCert::new().unwrap());
    //     cert.public();
    // }

    #[test]
    fn test_secret() {
        let cert = Cert::new(ZCert::new().unwrap());
        cert.secret();
    }

    #[test]
    fn test_header() {
        let cert = Cert::new(ZCert::new().unwrap());
        cert.header(true);
    }

    #[test]
    fn test_add_metadata() {
        let cert = Cert::new(ZCert::new().unwrap());
        let mut header = String::new();
        cert.add_metadata(&mut header);
    }
}
