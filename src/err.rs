/*!
An Error type.
*/
use std::{
    error::Error,
    fmt::{Display, Formatter},
    io,
    process::{Command, ExitStatus},
};

#[derive(Debug)]
pub struct YargErr(String);

impl Display for YargErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<std::fmt::Error> for YargErr {
    fn from(e: std::fmt::Error) -> Self {
        let msg = format!("fmt error: {}", &e);
        YargErr(msg)
    }
}

impl From<std::io::Error> for YargErr {
    fn from(e: std::io::Error) -> Self {
        let msg = format!("I/O error: {}", &e);
        YargErr(msg)
    }
}

impl Error for YargErr {}

fn format_cmd(prog: &Command) -> String {
    let mut line = prog.get_program().to_string_lossy().to_string();
    for arg in prog.get_args() {
        line.push(' ');
        line.push_str(&arg.to_string_lossy());
    }
    line
}

impl YargErr {
    pub fn new(msg: String) -> YargErr {
        YargErr(msg)
    }

    pub fn with_msg(prog: &Command, msg: String) -> YargErr {
        let msg = format!("{}: {}", format_cmd(prog), &msg);
        YargErr(msg)
    }

    pub fn exec_err(prog: &Command, e: io::Error) -> YargErr {
        let msg = format!("error spawning {}: {}", format_cmd(prog), &e);
        YargErr(msg)
    }

    pub fn exit_err(prog: &Command, s: ExitStatus) -> YargErr {
        let msg = match s.code() {
            Some(code) => format!("{} returned exit code {}", format_cmd(prog), &code),
            None => format!("{} exited with failure", format_cmd(prog)),
        };
        YargErr(msg)
    }
}
