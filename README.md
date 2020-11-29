# unix-exec-output-catcher
A library written in Rust that executes an executable in a child process and catches its output (stdout and stderr).

## ⚠️ Difference to std::process::Command 🚨
`std::process::Command` does the same in the standard library but **with one exception**:
My library gives you access to stdout, stderr, **and "stdcombined"**. This way you get all output
lines in the order they appeared. That's the unique feature of this crate.
[std/process/struct.Command.html#method.output](https://doc.rust-lang.org/std/process/struct.Command.html#method.output)

## TL;DR;
The call to `fork_exec_and_catch()` is blocking. If the program produces infinite output to
stdout or stderr, this function will never return. If the program produces 1GB of output
this function will consume 1GB of memory. See examples directory for example code.
Please read the warnings/information in `OCatchStrategy`. The strategy determines
in what way "STDCOMBINED" get's collected.

## Will my `"STDCOMBINED"` output (STDOUT + STDERR) be in right order?
* `OCatchStrategy::StdCombined`: definitely
* `OCatchStrategy::StdSeparately`: most probably, but there is no guarantee. If there are alternating
   prints to STDOUT/STDERR in a row without a few hundreds microseconds in between, it will probably 
   happen due to scheduling and in-kernel buffering that STDOUT/STDERR is not captured in correct
   order. 
See Rust comments for more information.

## Example
```rust
use unix_exec_output_catcher::{fork_exec_and_catch, OCatchStrategy};

fn main() {
    // executes "ls" with "-la" as argument.
    // this is equivalent to running "$ ls -la" in your shell.
    // The line by line output is stored inside the result.
    let res = fork_exec_and_catch(
        "ls", 
        vec!["ls", "-la"], 
        OCatchStrategy::StdSeparately
    );
    println!("{:#?}", res.unwrap());
}
```


## Used technologies / important keywords
- Unix (including but not limited to Linux-distributions, MacOS)
- `pipe()`
- `exec()`
- `fork()`
- `dup2()`
