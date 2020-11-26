# unix-exec-output-catcher
A library written in Rust that executes an executable in a child process and catches its output (stdout and stderr).

## TL;DR;
The call to `fork_exec_and_catch()` is blocking. If the program produces infinite output to
stdout or stderr, this function will never return. If the program produces 1GB of output
this function will consume 1GB of memory. See examples directory for example code.

## Example
```
use unix_exec_output_catcher::fork_exec_and_catch;

fn main() {
    // executes "ls" with the "-la" args.
    // this is equivalent to running "$ ls -la" in your shell.
    // The line by line output is stored inside the result.
    let res = fork_exec_and_catch("ls", vec!["ls", "-la"]);
    println!("{:#?}", res.unwrap());
}
```


## Used technologies / important keywords
- Unix (including but not limited to Linux-distributions, MacOS)
- `pipe()`
- `exec()`
- `fork()`
- `dup2()`
