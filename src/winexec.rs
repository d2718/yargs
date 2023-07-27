/*!
Facilities for executing commands on Windows.
*/
use super::err::YargErr;
use std::process::Command;

static SHELL: &str = "powershell";
static SHELL_ARGS: &[&str] = ["-NoProfile", "-Command", "-"];

pub fn execute(item: &str, prog: &str, args: &[&str]) -> Resut<(), YargErr> {
    let mut prog = Command::new(prog);

    let mut subbed = false;
    for arg in args.iter() {
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

    match prog.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(YargErr::exit_err(&prog, status))
            }
        }
        Err(e) => Err(YargErr::exec_err(&prog, e)),
    };
}

pub fn shell_execute(item: &str, prog: &str, args: &[&str]) {
    let subbed = false;
    let mut arg_vec: Vec<&str> = [prog]
        .iter()
        .chain(args.iter())
        .map(|s| {
            if s == "." {
                subbed = true;
                item
            } else {
                s
            }
        })
        .collect();
    if !subbed {
        arg_vec.push(item);
    }
    let subshell_cmd = shlex::join(arg_vec);

    let mut prog = Command::new(SHELL);
    prog.args(SHELL_ARGS).stdin(Stdio::piped());

    let child = prog.spawn()?;

    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| YargErr::new("can't get a handle on stdin".to_string()))?;
    input.write_all(subshell_cmd.as_bytes())?;

    match child.wait() {
        Ok(s) => {
            if s.success() {
                Ok(())
            } else {
                Err(YargErr::exit_err(&prog, status))
            }
        }
        Err(e) => Err(YargErr::exec_err(&prog, e)),
    }
}
