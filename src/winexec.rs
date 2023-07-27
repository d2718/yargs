/*!
Facilities for executing commands on Windows.
*/
use super::err::YargErr;
use shell_escape::windows::escape;
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
};

static SHELL: &str = "powershell";
static SHELL_ARGS: &[&str] = &["-NoProfile", "-Command", "-"];

pub fn execute(item: &str, prog: &str, args: &[&str]) -> Result<(), YargErr> {
    let mut prog = Command::new(prog);

    let mut subbed = false;
    for arg in args.iter() {
        if arg == &"." {
            prog.arg(item);
            subbed = true;
        } else {
            prog.arg(arg);
        }
    }
    if !subbed {
        prog.arg(item);
    }

    match prog.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(YargErr::exit_err(&prog, status))
            }
        }
        Err(e) => Err(YargErr::exec_err(&prog, e)),
    }
}

pub fn shell_execute(item: &str, prog: &str, args: &[&str]) -> Result<(), YargErr> {
    let mut subbed = false;
    let mut arg_vec: Vec<_> = [prog]
        .iter()
        .chain(args.iter())
        .map(|&s| {
            if s == "." {
                subbed = true;
                escape(Cow::from(item))
            } else if s == "|" {
                Cow::from("|")
            } else {
                escape(Cow::from(s))
            }
        })
        .collect();
    if !subbed {
        arg_vec.push(Cow::from(item));
    }
    let subshell_cmd = arg_vec.join(" ");

    let mut prog = Command::new(SHELL);
    prog.args(SHELL_ARGS).stdin(Stdio::piped());

    let mut child = prog.spawn()?;

    /*
    We need this scope here so that `child`'s stdin will get dropped.
    Otherwise powershell will just wait to keep reading from stdin forever.
    */
    {
        let mut input = child
            .stdin
            .take()
            .ok_or_else(|| YargErr::new("can't get a handle on stdin".to_string()))?;
        input.write_all(subshell_cmd.as_bytes())?;
        input.flush()?;
    }

    match child.wait() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(YargErr::exit_err(&prog, status))
            }
        }
        Err(e) => Err(YargErr::exec_err(&prog, e)),
    }
}
