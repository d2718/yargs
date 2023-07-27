/*!
Because yargs involves entering command-line options for _another_ program
on its command line, some special argument parsing is in order.
*/
use std::ffi::OsString;

static DEFAULT_FENCE: &str = r#"\r?\n"#;

/*
THE FOLLOWING WORKAROUND HAS BEEN TEMPORARILY REMOVED, and replaced by
a different workaround. Currently all arguments must be valid UTF-8 Strings;
eventually this may get switched back to the more relaxed OsString, so
this code is staying here for the time being.

***

Okay, so this is slightly annoying.

It's not possible to use OsStr literals in match patterns because OsStr
literals don't exist. The standard way to match an OsStr is to cast it
to either a byte slice or a string slice and match against _those_
literals.

HOWEVER, an OsStr can't be cast to a byte slice on Windows because
Windows uses `&[u16]` as the backing store for it! UTF-16!!! AND,
casting to an `&str` would mean choking on non-UTF-8 data that may
nonetheless be a valid OsStr.

So our solution here is to define an enum for the different possible
flag/option tokens, build a HashMap of OsStrs to those enum values,
and then match on the enums.This is maybe awkward and inelegant,
but it's portable and doesn't require any #[cfg(<os_type>)] flags
(at least not on my part).

#[derive(Clone, Copy, Default)]
enum ArgOpt {
    Hyphens,
    Delimiter,
    Subshell,
    Continue,
    Help,
    Version,
    #[default]
    None,
}

static OPT_MAP: &[(ArgOpt, &[&str])] = &[
    (ArgOpt::Hyphens, &["--"]),
    (ArgOpt::Delimiter, &["-d", "--delim", "--delimiter"]),
    (
        ArgOpt::Subshell,
        &["-s", "--sub", "--sh", "--shell", "--subshell"],
    ),
    (ArgOpt::Continue, &["-c", "--cont", "--continue"]),
    (ArgOpt::Help, &["-h", "--help"]),
    (ArgOpt::Version, &["-V", "--version"]),
];

// Generates the HashMap used for flag/option lookup in the
// command-line-argument parsing loop.
fn make_opt_map() -> HashMap<OsString, ArgOpt> {
    let mut m: HashMap<OsString, ArgOpt> = HashMap::new();

    for &(opt, flags) in OPT_MAP.iter() {
        for f in flags.iter() {
            m.insert(OsString::from(f), opt);
        }
    }

    m
}
*/

enum ArgMode {
    Positional,
    PostPositional,
    Fence,
}

pub struct Opts {
    /// Name of program/shell command.
    /// Should be first positional argument.
    pub exec: Option<String>,
    /// Arguments to program/shell command.
    /// The rest of the positional arguments.
    pub args: Vec<String>,
    /// Pattern for the regex used to delimit items.
    /// Should default to "\r?\n" (X-platform newline).
    pub fence: String,
    /// Run commands in a subshell.
    pub subshell: bool,
    /// Whether to continue when an error is encountered.
    pub cont: bool,
    /// Instruction to print the help info.
    pub help: bool,
    /// Instruction to print version information.
    pub version: bool,
}

impl Opts {
    pub fn parse() -> Result<Opts, OsString> {
        let mut exec: Option<String> = None;
        let mut args: Vec<String> = Vec::new();
        let mut fence: Option<String> = None;
        let mut cont = false;
        let mut subshell = false;
        let mut help = false;
        let mut version = false;
        let mut mode = ArgMode::Positional;

        /*
        Temporarily, but possibly permanently, removed. See long rant
        starting on line 10.

        ***

        let opt_map = make_opt_map();
        */

        for arg in std::env::args_os().skip(1) {
            let arg = arg.into_string()?;
            match mode {
                ArgMode::PostPositional => args.push(arg),
                ArgMode::Positional => match arg.as_str() {
                    "--" => {
                        mode = ArgMode::PostPositional;
                    }
                    "-d" | "--delim" | "--delimiter" => {
                        if fence.is_none() {
                            mode = ArgMode::Fence;
                        } else {
                            args.push(arg);
                        }
                    }
                    "-s" | "--sh" | "--sub" | "--shell" | "--subshell" => {
                        if subshell {
                            args.push(arg);
                        } else {
                            subshell = true;
                        }
                    }
                    "-c" | "--cont" | "--continue" => {
                        if cont {
                            args.push(arg);
                        } else {
                            cont = true;
                        }
                    }
                    "-h" | "--help" => {
                        help = true;
                    }
                    "-V" | "--version" => {
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
                    fence = Some(arg);
                }
            }
        }

        let fence = fence.unwrap_or_else(|| String::from(DEFAULT_FENCE));

        Ok(Opts {
            exec,
            args,
            fence,
            cont,
            subshell,
            help,
            version,
        })
    }
}
