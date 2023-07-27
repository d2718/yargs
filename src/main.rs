pub mod err;
#[cfg(unix)]
pub mod exec;
pub mod opt;
#[cfg(windows)]
pub mod winexec;

use std::{io::stdin, process::exit};

// use bstr::ByteSlice;
use regex_chunker::ByteChunker;

#[cfg(unix)]
use exec::{execute, shell_execute};
use opt::Opts;
#[cfg(windows)]
use winexec::{execute, shell_execute};

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP: &str = r#"A friendlier xargs. Also a piratical exclamation.

Usage: yargs [ OPTIONS ] <CMD> [ ARGS... ]

Arguments:
  <CMD> Command to execute for each item of input
  [ARGS...] Additional arguments to <CMD>

Options:
  -d, --delimiter <DELIM>  Regex to delimit input items
                           (default is "\r?\n")
  -s, --subshell           Run each command in a subshell
                           (default is to invoke directly)
  -c, --continue           Continue and ignore errors
                           (default is to halt upon error)
  -h, --help               Print this message
  -V, --version            Print version information"#;

fn print_version() {
    println!("yargs {}", VERSION);
}

fn print_help() {
    println!("{}", HELP);
}

fn main() -> Result<(), String> {
    let opts =
        Opts::parse().map_err(|e| format!("argument not valid UTF-8: {}", &e.to_string_lossy()))?;

    if opts.help {
        print_help();
        exit(0);
    } else if opts.version {
        print_version();
        exit(0);
    }

    let exec = match opts.exec {
        Some(exec) => exec,
        None => {
            eprintln!("Must supply command to execute.");
            exit(2);
        }
    };
    let args: Vec<&str> = opts.args.iter().map(String::as_str).collect();

    for item in ByteChunker::new(stdin(), &opts.fence).unwrap() {
        let item = item.map_err(|e| format!("{}", &e))?;
        let item = String::from_utf8_lossy(&item);
        let res = if opts.subshell {
            shell_execute(&item, &exec, &args)
        } else {
            execute(&item, &exec, &args)
        };
        if !opts.cont && res.is_err() {
            res.map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
