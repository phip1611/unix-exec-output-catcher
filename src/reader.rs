//! Abstraction and functions related to the reading of the output.

use crate::ProcessOutput;
use crate::error::UECOError;
use crate::pipe::{Pipe, CatchPipes};
use std::rc::Rc;
use std::cell::RefCell;
use crate::child::{ChildProcess, ProcessState};
use std::thread;

/// Read all content from the child process output
/// as long as it's running. Catches STDOUT and STDERR.
/// This is the generic interface. Implementation
/// depends on the strategy.
pub trait OutputReader {

    /// Reads all output in a blocking way as long as
    /// the child process is running.
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError>;

}

/// Reader for `[crate::OCatchStrategy::StdCombined]`.
/// Catches `"STDCOMBINED"` in right order but `STDOUT`
/// and `STDERR` not at all.
// #[derive(Debug)]
pub struct SimpleOutputReader<'a> {
    pipe: Rc<RefCell<Pipe>>,
    child: &'a mut ChildProcess,
}

impl <'a> SimpleOutputReader<'a> {
    pub fn new(child: &'a mut ChildProcess) -> Self {
        // in this case stdout and stderr both use the same pipe
        SimpleOutputReader { pipe: child.stdout_pipe().clone(), child }
    }
}

impl <'a> OutputReader for SimpleOutputReader<'a> {
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError> {
        let pipe = self.pipe.borrow();
        let mut lines = vec![];

        let mut eof = false;
        loop {
            let line = pipe.read_line()?;
            match line {
                None => { eof = true }
                Some(line) => {
                    eof = false;
                    lines.push(line)
                }
            }

            let process_is_running = self.child.check_state_nbl() == ProcessState::Running;
            let process_finished = !process_is_running;
            if process_finished && eof {
                trace!("Child finished & read EOF");
                break;
            }
        }

        let lines = lines.into_iter().map(|s| Rc::new(s)).collect();
        let output = ProcessOutput::new(
            None,
            None,
            Some(lines),
            self.child.exit_code().unwrap(),
        );
        Ok(output)
    }
}

/// Reader for `[crate::OCatchStrategy::StdSeparately]`.
/// Catches `STDOUT` and `STDERR`, but the order of
/// `"STDCOMBINED"` is only maybe correct.
// #[derive(Debug)]
pub struct SimultaneousOutputReader<'a>  {
    stdout_pipe: Rc<RefCell<Pipe>>,
    stderr_pipe: Rc<RefCell<Pipe>>,
    child: &'a mut ChildProcess,
}

impl <'a> SimultaneousOutputReader<'a> {
    pub fn new(child: &'a mut ChildProcess) -> Self {
        SimultaneousOutputReader {
            stdout_pipe: child.stdout_pipe().clone(),
            stderr_pipe: child.stderr_pipe().clone(),
            child
        }
    }
}

impl <'a> OutputReader for SimultaneousOutputReader<'a> {
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError> {
        let stdout_t = thread::spawn(move || {

        });
        let stderr_t = thread::spawn(move || {

        });
        unimplemented!()
    }
}
