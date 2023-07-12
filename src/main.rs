mod iter;
mod opt;

use std::{
    ffi::OsStr,
    fmt::Write,
    io::stdin,
    process::{exit, Command},
};

use iter::RegexChunker;
use opt::Opts;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP: &str = r#"A friendlier xargs. Also a piratical exclamation.

Usage: yargs [ OPTIONS ] <CMD> [ ARGS... ]

Arguments:
  <CMD> Command to execute for each item of input
  [ARGS...] Additional arguments to <CMD>

Options:
  -d, --delimiter <DELIM>  Regex to delimit input items
                           (default is "\r?\n")
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

/// Write the command an arguments passed to `cmd`.
/// This is for reporting errors.
fn write_command_line<W: Write>(mut buff: W, cmd: &Command) -> std::fmt::Result {
    write!(buff, "\"{}\"", &cmd.get_program().to_string_lossy())?;
    for arg in cmd.get_args() {
        write!(buff, " \"{}\"", &arg.to_string_lossy())?;
    }
    Ok(())
}

/// Execute the command line whose args are in `cmd`. If one of the args is
/// a bare '.', replace it with `item`; otherwise, insert `item` at the end.
fn execute<S: AsRef<OsStr>>(
    item: &OsStr,
    exec: &OsStr,
    args: &[S],
    cont: bool,
) -> Result<(), String> {
    let mut prog = Command::new(exec);

    let mut subbed = false;
    for arg in args.iter() {
        let arg = arg.as_ref();
        if arg == "." {
            prog.arg(item);
            subbed = true;
        } else {
            prog.arg(arg);
        }
    }
    if !subbed {
        prog.arg(item);
    }

    let status = match prog.status() {
        Ok(s) => s,
        Err(e) => {
            let mut err_msg = "error running ".to_string();
            write_command_line(&mut err_msg, &prog).map_err(|e| format!("{}", &e))?;
            write!(&mut err_msg, " :{}", &e).map_err(|e| format!("{}", &e))?;
            if cont {
                eprintln!("{}", &err_msg);
                return Ok(());
            } else {
                return Err(err_msg);
            }
        }
    };

    if status.success() {
        Ok(())
    } else {
        let mut err_msg = String::new();
        write_command_line(&mut err_msg, &prog).map_err(|e| format!("{}", &e))?;
        match status.code() {
            Some(code) => write!(&mut err_msg, " returned exit code {}", code)
                .map_err(|e| format!("{}", &e))?,
            None => write!(&mut err_msg, " exited with failure").map_err(|e| format!("{}", &e))?,
        }
        if cont {
            eprintln!("{}", &err_msg);
            Ok(())
        } else {
            Err(err_msg)
        }
    }
}

fn main() -> Result<(), String> {
    let opts = Opts::parse()?;

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

    for item in RegexChunker::new(stdin(), &opts.fence).unwrap() {
        let item = item.unwrap();
        execute(&item, &exec, &opts.args, opts.cont)?;
    }

    Ok(())
}
