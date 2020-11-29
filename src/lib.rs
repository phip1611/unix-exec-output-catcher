//! This library lets you execute a child process and catch its output (stdout and stderr).
//! This is useful if you want to use the output from a specific command and transform it
//! in your program.
//!
//! ⚠️ Difference to std::process::Command 🚨
//! `std::process::Command` does the same in the standard library but **with one exception**:
//! My library gives you access to `stdout`, `stderr`, **and** `"stdcombined"`. This way you get all
//! output lines in the order they appeared. That's the unique feature of this crate.

use std::rc::Rc;
use derive_more::Display;

#[macro_use]
extern crate log;

mod error;
mod pipe;
mod libc_util;
mod exec;
mod child;
mod reader;

pub use exec::fork_exec_and_catch;

/// Holds the information from the executed process.
/// It depends on the `strategy` option of `[crate::fork_exec_and_catch]` whether
/// * `stdout_lines` and `stderr_lines` are correct but `stdcombined_lines` is only
///   maybe in correct order
/// * or `stdout_lines` and `stderr_lines` are `None`, but `stdcombined_lines` is in correct order
#[derive(Debug)]
pub struct ProcessOutput {
    exit_code: i32,
    stdout_lines: Option<Vec<Rc<String>>>,
    stderr_lines: Option<Vec<Rc<String>>>,
    stdcombined_lines: Option<Vec<Rc<String>>>,
}

impl ProcessOutput {

    /// Constructor.
    fn new(stdout_lines: Option<Vec<Rc<String>>>,
           stderr_lines: Option<Vec<Rc<String>>>,
           stdcombined_lines: Option<Vec<Rc<String>>>,
           exit_code: i32) -> Self {
        Self {
            stdout_lines,
            stderr_lines,
            stdcombined_lines,
            exit_code,
        }
    }

    pub fn stdout_lines(&self) -> Option<&Vec<Rc<String>>> {
        self.stdout_lines.as_ref()
    }
    pub fn stderr_lines(&self) -> Option<&Vec<Rc<String>>> {
        self.stderr_lines.as_ref()
    }
    pub fn stdcombined_lines(&self) -> Option<&Vec<Rc<String>>> {
        self.stdcombined_lines.as_ref()
    }
}

#[derive(Debug, Display, Copy, Clone)]
pub enum OCatchStrategy {
    /// Catches all output lines of STDOUT and STDERR in correct order on a line
    /// by line base but there is no way to find out STDOUT-only or STDERR-only lines.
    StdCombined,
    /// Catches all output lines from STDOUT and STDERR separately. There is also a
    /// "STDCOMBINED" vector, but the order is not 100% correct. It's only approximately correct.
    StdSeparately,
}

#[cfg(test)]
mod tests {

    use super::*;

    // RUst tests doesn't work with fork, dup2 and other fun :)
    // weird output.. use the test binary instead!
    /*#[test]
    fn test_fork_exec_and_catch() {
        let res = fork_exec_and_catch("echo", vec!["echo", "hallo"]);

        println!("{:#?}", res);
        // fork_exec_and_catch("sysctl", vec!["sysctl", "-a"]);
    }*/
}
