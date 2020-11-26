# unix-exec-output-catcher
A library written in Rust that executes an executable in a child process and catches its output (stdout and stderr).

## TL;DR;
The call to `fork_exec_and_catch()` is blocking. If the program produces infinite output to
stdout or stderr, this function will never return. If the program produces 1GB of output
this function will consume 1GB of memory.

## Used technologies / important keywords
- Unix (including but not limited to Linux-distributions, MacOS)
- `pipe()`
- `exec()`
- `fork()`
- `dup2()`
