//! This library lets you execute a child process and catch its output (stdout and stderr).
//! This is useful if you want to use the output from a specific command and transform it
//! in your program.

use std::rc::Rc;

#[macro_use]
extern crate log;

mod error;
mod pipe;
mod libc_util;
mod exec;

pub use exec::fork_exec_and_catch;

/// Holds the text output lines for stdout
/// and stderr of the executed child process.
/// The stdcombined-property holds both combined
/// in the order they appeared.
#[derive(Debug)]
pub struct ProcessOutput {
    stdout_lines: Vec<Rc<String>>,
    stderr_lines: Vec<Rc<String>>,
    // combines values from both in the
    // order they occurred
    stdcombined_lines: Vec<Rc<String>>,
}

impl ProcessOutput {

    /// Constructor.
    fn new(stdout_lines: Vec<Rc<String>>,
               stderr_lines: Vec<Rc<String>>,
               stdcombined_lines: Vec<Rc<String>>) -> Self {
        Self {
            stdout_lines,
            stderr_lines,
            stdcombined_lines
        }
    }

    pub fn stdout_lines(&self) -> &Vec<Rc<String>> {
        &self.stdout_lines
    }
    pub fn stderr_lines(&self) -> &Vec<Rc<String>> {
        &self.stderr_lines
    }
    pub fn stdcombined_lines(&self) -> &Vec<Rc<String>> {
        &self.stdcombined_lines
    }
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
