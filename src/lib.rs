//! This library lets you execute a child process and catch its output (stdout and stderr).
//! This is useful if you want to use the output from a specific command and transform it
//! in your program.
//!
//! ⚠️ Difference to std::process::Command 🚨
//! `std::process::Command` does the same in the standard library but **with one exception**:
//! My library gives you access to `stdout`, `stderr`, **and** `"stdcombined"`. This way you get all
//! output lines in the order they appeared. That's the unique feature of this crate.

use derive_more::Display;
use std::rc::Rc;

#[macro_use]
extern crate log;

mod child;
pub mod error;
mod exec;
mod libc_util;
mod pipe;
mod reader;

pub use exec::fork_exec_and_catch;

/// Holds the information from the executed process. It depends on the `strategy` option of
/// [`crate::fork_exec_and_catch`] how the output is structured.
///
/// The strategy results in the following two kinds of outputs:
/// * `stdout_lines` and `stderr_lines` are correct but `stdcombined_lines` is only
///   maybe in correct order
/// * or `stdout_lines` and `stderr_lines` are `None`, but `stdcombined_lines` is in correct order
#[derive(Debug)]
pub struct ProcessOutput {
    /// Exit code of the process. 0 is success, >1 is error.
    /// See https://man7.org/linux/man-pages/man3/errno.3.html
    exit_code: i32,
    /// * `None` for [`crate::OCatchStrategy::StdCombined`]
    /// * `Some` for [`crate::OCatchStrategy::StdSeparately`]
    stdout_lines: Option<Vec<Rc<String>>>,
    /// * `None` for [`crate::OCatchStrategy::StdCombined`]
    /// * `Some` for [`crate::OCatchStrategy::StdSeparately`]
    stderr_lines: Option<Vec<Rc<String>>>,
    /// * All output lines in correct order for [`crate::OCatchStrategy::StdCombined`]
    /// * All output lines in not guaranteed correct order for [`crate::OCatchStrategy::StdSeparately`]
    stdcombined_lines: Vec<Rc<String>>,
    /// The strategy that was used. See [`crate::OCatchStrategy::StdSeparately`].
    strategy: OCatchStrategy,
}

impl ProcessOutput {
    /// Constructor.
    fn new(
        stdout_lines: Option<Vec<Rc<String>>>,
        stderr_lines: Option<Vec<Rc<String>>>,
        stdcombined_lines: Vec<Rc<String>>,
        exit_code: i32,
        strategy: OCatchStrategy,
    ) -> Self {
        Self {
            stdout_lines,
            stderr_lines,
            stdcombined_lines,
            exit_code,
            strategy,
        }
    }

    /// Getter for `stdout_lines`. This is only available if [`OCatchStrategy::StdSeparately`] was used.
    pub fn stdout_lines(&self) -> Option<&Vec<Rc<String>>> {
        self.stdout_lines.as_ref()
    }
    /// Getter for `stderr_lines`. This is only available if [`OCatchStrategy::StdSeparately`] was used.
    pub fn stderr_lines(&self) -> Option<&Vec<Rc<String>>> {
        self.stderr_lines.as_ref()
    }
    /// Getter for `stdcombined_lines`. The correctness of the ordering depends on the used [`OCatchStrategy`].
    pub fn stdcombined_lines(&self) -> &Vec<Rc<String>> {
        &self.stdcombined_lines
    }
    /// Getter for `exit_code` of the executed child process.
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
    /// Getter for the used [`OCatchStrategy`].
    pub fn strategy(&self) -> OCatchStrategy {
        self.strategy
    }
}

/// Determines the strategy that is used to get STDOUT, STDERR, and "STDCOMBINED".
/// Both has advantages and disadvantages.
#[derive(Debug, Display, Copy, Clone)]
pub enum OCatchStrategy {
    /// Catches all output lines of STDOUT and STDERR in correct order on a line
    /// by line base. There is no way to find out STDOUT-only or STDERR-only lines.
    StdCombined,
    /// Catches all output lines from STDOUT and STDERR separately. There is also a
    /// "STDCOMBINED" vector, but the order is not 100% correct.  It's only approximately correct
    /// on a best effort base. If between each STDOUT/STDERR-alternating output is ≈100µs
    /// (a few thousand cycles) it should be definitely fine, but there is no guarantee for that.
    /// Also the incorrectness is not deterministic. This is because
    /// STDOUT and STDERR are two separate streams. Scheduling and buffering result in
    /// different results.
    StdSeparately,
}

#[cfg(test)]
mod tests {

    // use super::*;

    // RUst tests doesn't work with fork, dup2 and other fun :)
    // weird output.. use the test binary instead!
    /*#[test]
    fn test_fork_exec_and_catch() {
        let res = fork_exec_and_catch("echo", vec!["echo", "hallo"]);

        println!("{:#?}", res);
        // fork_exec_and_catch("sysctl", vec!["sysctl", "-a"]);
    }*/
}
