# yargs

```text
A friendlier xargs. Also a piratical exclamation.

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
  -V, --version            Print version information
```

## Installation

It should just `cargo build --release`.

## Use

Like `xargs`, `yargs` reads items from stdin and executes the supplied
command once for each argument.

By default, each item is inserted at the end of the supplied command line.

```text
$ ls src/* | yargs wc -l
141 src/iter.rs
118 src/main.rs
90 src/opt.rs
```

The item can be inserted elsewhere by including a bare `.` in the command.

```text
$ ls src/* | yargs cp . /mnt/sda1/src
```

Use the standard `--` to ensure all arguments that follow are passed on
correctly (for commands that use the same options as `yargs`).

```text
$ ls src/* | yargs wc -- -c
4535 src/iter.rs
3134 src/main.rs
2794 src/opt.rs
```

If you've already used one of `yargs`'s flags, it'll automatically get
passed on if you specify it again.

```text
$ ls | target/debug/yargs -c wc -c
1324 Cargo.lock
316 Cargo.toml
1060 LICENSE
1542 README.md
wc: src: Is a directory
0 src
"wc" "-c" "src" returned exit code 1
wc: target: Is a directory
0 target
"wc" "-c" "target" returned exit code 1
wc: test: Is a directory
0 test
"wc" "-c" "test" returned exit code 1
```

Specify an alternate regular expression to separate items with `-d`.
(The default is `\r?\n`, the "cross-platform newline".)

## Windows Quirks

### PowerShell is not particularly composable

This program will run under 64-bit Windows, doing what it says on the tin,
but is awkward because Powershell's built-in listing facilities generate
a lot of noise by default:

```text
PS C:\Users\19195\dev\yargs> ls


    Directory: C:\Users\19195\dev\yargs


    Mode                 LastWriteTime         Length Name
    ----                 -------------         ------ ----
    d-----         7/12/2023  11:39 AM                src
    d-----         7/12/2023  11:43 AM                target
    d-----         7/12/2023  10:31 AM                test
    -a----         7/12/2023  10:31 AM            442 .gitignore
    -a----         7/12/2023  11:43 AM           1324 Cargo.lock
    -a----         7/12/2023  12:10 PM            316 Cargo.toml
    -a----         7/12/2023  10:31 AM           1060 LICENSE
    -a----         7/12/2023  10:31 AM           1941 README.md
    
```

You can get around this particular problem with `Get-ChildItem -name`,

```text
PS C:\Users\19195\dev\yargs> Get-ChildItem -name
src
target
test
.gitignore
Cargo.lock
Cargo.toml
LICENSE
README.md
```

or better yet with a
[`find`](https://www.gnu.org/software/findutils/manual/html_mono/find.html)-like
utility for Windows (~~if one exists~~
oh [one definitely does](https://github.com/d2718/fine)).

However, a lot of facilities that are supplied by programs in a Unix-style
environment are "PowerShell Cmdlets" that can't be run like a program:

```text
PS C:\Users\19195\dev\yargs> Get-ChildItem -file -name | target/debug/yargs Copy-Item . scratch
Error: "error running \"Copy-Item\" \".gitignore\" \"scratch\" :program not found"
```

__It works fine on WSL2.__

### UTF-16

Windows plumbing uses UTF-16 for some reason, so `OsStrings` (and `&OsStr`s)
don't convert nicely to byte slices (`&[u16]`!! blech!). ~~The easiest thing
for the short term was to just convert all input into `String` under Windows.
So all inputs and command-line arguments have to be valid Unicode under
Windows.~~ Actually, I think I got this figured out, and ended up being
able to remove the `#[cfg(windows)]` and `#[cfg(unix)]` stuff. The custom
regex delimiter still has to be converted to a string, but that's the
case on all platforms.

## Issues / To Do

Add support for `\1`, `\2`, &c. syntax in command line to allow the use of
multiple items in a single command.

Reduce the error-handling ad-hockery.
