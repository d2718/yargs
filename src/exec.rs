/*!
Facilities for executing commands.
*/
use super::err::YargErr;
use std::process::Command;

static SHELL: &str = "sh";
static SHELL_ARG: &str = "-c";

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
    let mut arg_vec: Vec<&str> = [prog]
        .iter()
        .chain(args.iter())
        .map(|s| {
            if s == &"." {
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
    prog.args([SHELL_ARG, &subshell_cmd]);

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
