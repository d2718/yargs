/**!
Because yargs involves entering command-line options for _another_ program
on its command line, some special argument parsing is in order.
*/
use std::{ffi::OsString, os::unix::ffi::OsStrExt};

enum ArgMode {
    Positional,
    PostPositional,
    Fence,
}

pub struct Opts {
    /// Name of program/shell command.
    /// Should be first positional argument.
    pub exec: Option<OsString>,
    /// Arguments to program/shell command.
    /// The rest of the positional arguments.
    pub args: Vec<OsString>,
    /// Pattern for the regex used to delimit items.
    /// Should default to "\r?\n" (X-platform newline).
    pub fence: String,
    /// Whether to continue when an error is encountered.
    pub cont: bool,
    /// Instruction to print the help info.
    pub help: bool,
    /// Instruction to print version information.
    pub version: bool,
}

impl Opts {
    pub fn parse() -> Result<Opts, String> {
        let mut exec: Option<OsString> = None;
        let mut args: Vec<OsString> = Vec::new();
        let mut fence = r"\r?\n".to_string();
        let mut cont = false;
        let mut help = false;
        let mut version = false;
        let mut mode = ArgMode::Positional;

        for arg in std::env::args_os().skip(1) {
            match mode {
                ArgMode::PostPositional => args.push(arg),
                ArgMode::Positional => match arg.as_bytes() {
                    b"--" => {
                        mode = ArgMode::PostPositional;
                    }
                    b"-d" | b"--delim" | b"--delimiter" => {
                        mode = ArgMode::Fence;
                    }
                    b"-c" | b"--cont" | b"--continue" => {
                        cont = true;
                    }
                    b"-h" | b"--help" => {
                        help = true;
                    }
                    b"-V" | b"--version" => {
                        version = true;
                    }
                    _ => {
                        if exec.is_none() {
                            exec = Some(arg);
                        } else {
                            args.push(arg);
                        }
                    }
                },
                ArgMode::Fence => {
                    if let Some(dstr) = arg.to_str() {
                        fence = dstr.to_string();
                    } else {
                        return Err(format!(
                            "invalid delimiter regex: {}",
                            &arg.to_string_lossy()
                        ));
                    }
                }
            }
        }

        Ok(Opts {
            exec,
            args,
            fence,
            cont,
            help,
            version,
        })
    }
}
