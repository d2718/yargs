use std::{
    ffi::{OsStr, OsString},
    fmt::{write, Write},
    process::Command,
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Opts {
    /// Command to run for each input item.
    #[arg(value_name = "command")]
    cmd: Vec<OsString>,

    /// Input item delimiter (default is a newline).
    #[arg(short, long)]
    delim: Option<OsString>,

    /// Continue on error (default is to stop).
    #[arg(short, long = "continue")]
    cont: bool,
}

fn write_command_line<W: Write>(mut buff: W, cmd: &Command) -> std::fmt::Result {
    write!(buff, "\"{}\"", &cmd.get_program().to_string_lossy())?;
    for arg in cmd.get_args() {
        write!(buff, " \"{}\"", &arg.to_string_lossy())?;
    }
    Ok(())
}

fn execute<S: AsRef<OsStr>>(item: &OsStr, cmd: &[S]) -> Result<(), String> {
    let exec = cmd.first().unwrap().as_ref();
    let mut prog = Command::new(exec);

    let mut subbed = false;
    for arg in &cmd[1..] {
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
            return Err(err_msg);
        }
    };

    if status.success() {
        return Ok(());
    } else {
        let mut err_msg = String::new();
        write_command_line(&mut err_msg, &prog).map_err(|e| format!("{}", &e))?;
        match status.code() {
            Some(code) => write!(&mut err_msg, " returned exit code {}", code)
                .map_err(|e| format!("{}", &e))?,
            None => write!(&mut err_msg, " exited with failure").map_err(|e| format!("{}", &e))?,
        }
        return Err(err_msg);
    }
}

fn main() {
    let opts = Opts::parse();

    println!("{:?}", &opts);
}
