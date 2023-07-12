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
  -c, --continue           Continue and ignore errors
                           (default is to halt upon error)
  -h, --help               Print this message
  -V, --version            Print version information
```

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

Specify an alternate regular expression to separate items with `-d`.
(The default is `\r?\n`, the "cross-platform newline".)

## Issues / To Do

Implement the `-c` option.

Investigate and support Windows (works fine on WSL2).

Add support for `\1`, `\2`, &c. syntax in command line to allow the use of
multiple items in a single command.

Reduce the error-handling ad-hockery.
