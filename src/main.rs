// Copyright 2019 Stephen Connolly.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE.txt or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT.txt or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate core;
extern crate getopts;
extern crate regex;

use core::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::io;
use std::io::BufRead;
use std::io::LineWriter;
use std::io::Write;

use getopts::Options;
use regex::Captures;
use regex::Regex;

fn create_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu and exit");
    opts.optflag("V", "version", "print the version and exit");
    opts.optmulti(
        "v",
        "variable",
        "restrict expansion to named variables only",
        "VAR",
    )
    .optopt(
        "p",
        "prefix",
        "set the expansion prefix marker (default: ${)",
        "PREFIX",
    )
    .optopt(
        "s",
        "suffix",
        "set the expansion suffix marker (default: })",
        "PREFIX",
    );
    opts
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
    println!();
    println!("Rewrites input to output expanding environment variables");
    println!();
    println!("NOTE: Only ${{ENV_VAR}} and ${{ENV_VAR-default value}} are supported");
    println!("      (and you are on your own if your default value needs to include");
    println!("      a }} character)");
}

fn main() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let opts = create_options();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("V") {
        println!("{}", VERSION);
        return;
    }
    let prefix = match matches.opt_str("p") {
        Some(v) => v,
        None => "${".to_string(),
    };
    let suffix = match matches.opt_str("s") {
        Some(v) => v,
        None => "}".to_string(),
    };
    let vars = match matches.opt_present("v") {
        true => {
            let mut vars = HashMap::new();
            for var_name in matches.opt_strs("v") {
                vars.insert(
                    var_name.clone(),
                    match env::var(var_name.clone()) {
                        Ok(v) => (
                            Regex::new(
                                format!(
                                    r#"{}{}((:?-)(.*?))??{}"#,
                                    regex::escape(prefix.clone().as_str()),
                                    regex::escape(var_name.as_str()),
                                    regex::escape(suffix.clone().as_str())
                                )
                                .as_str(),
                            )
                            .unwrap(),
                            Some(v),
                        ),
                        Err(_) => (
                            Regex::new(
                                format!(
                                    r#"{}{}((:?-)(.*?))??{}"#,
                                    regex::escape(prefix.clone().as_str()),
                                    regex::escape(var_name.as_str()),
                                    regex::escape(suffix.clone().as_str())
                                )
                                .as_str(),
                            )
                            .unwrap(),
                            None,
                        ),
                    },
                );
            }
            vars
        }
        false => {
            let mut vars = HashMap::new();
            for (key, value) in env::vars() {
                vars.insert(
                    key.clone(),
                    (
                        Regex::new(
                            format!(
                                r#"{}{}((:?-)(.*?))??{}"#,
                                regex::escape(prefix.clone().as_str()),
                                regex::escape(key.as_str()),
                                regex::escape(suffix.clone().as_str())
                            )
                            .as_str(),
                        )
                        .unwrap(),
                        Some(value),
                    ),
                );
            }
            vars
        }
    };

    let reader = io::stdin();
    let mut writer = LineWriter::new(io::stdout());

    for line in reader.lock().lines() {
        let mut out = line.unwrap();
        for (_, (regex, value)) in &vars {
            let val = &value.clone();
            out = regex
                .replace_all(out.as_str(), |caps: &Captures| match caps.get(2) {
                    Some(mat) => match mat.as_str() {
                        ":-" => match val.borrow() {
                            Some(v) => {
                                if v.is_empty() {
                                    caps.get(3).map_or("", |m| m.as_str())
                                } else {
                                    v.as_str()
                                }
                            }
                            None => caps.get(3).map_or("", |m| m.as_str()),
                        },
                        _ => match val.borrow() {
                            Some(v) => v.as_str(),
                            None => caps.get(3).map_or("", |m| m.as_str()),
                        },
                    },
                    None => match val.borrow() {
                        Some(v) => v.as_str(),
                        None => caps.get(0).unwrap().as_str(),
                    },
                })
                .to_string();
        }
        writer.write(out.as_bytes()).unwrap();
        writer.write("\n".as_bytes()).unwrap();
        writer.flush().unwrap();
    }
}
