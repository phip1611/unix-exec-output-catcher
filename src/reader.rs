//! Abstraction and functions related to the reading of the output.

use crate::child::{ChildProcess, ProcessState};
use crate::error::UECOError;
use crate::pipe::Pipe;
use crate::{OCatchStrategy, ProcessOutput};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Read all content from the child process output
/// as long as it's running. Catches STDOUT and STDERR.
/// This is the generic interface. Implementation
/// depends on the strategy.
pub trait OutputReader {
    /// Reads all output in a blocking way as long as
    /// the child process is running.
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError>;

    /// The strategy this reader is responsible for.
    fn strategy() -> OCatchStrategy;
}

/// Reader for [`crate::OCatchStrategy::StdCombined`].
/// Catches `"STDCOMBINED"` in right order but `STDOUT`
/// and `STDERR` not at all.
// #[derive(Debug)]
pub struct SimpleOutputReader<'a> {
    pipe: Arc<Mutex<Pipe>>,
    child: &'a mut ChildProcess,
}

impl<'a> SimpleOutputReader<'a> {
    pub fn new(child: &'a mut ChildProcess) -> Self {
        // in this case stdout and stderr both use the same pipe
        SimpleOutputReader {
            pipe: child.stdout_pipe().clone(),
            child,
        }
    }
}

impl<'a> OutputReader for SimpleOutputReader<'a> {
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError> {
        let pipe = self.pipe.lock().unwrap();
        let mut lines = vec![];

        let mut eof;
        loop {
            let line = pipe.read_line()?;
            match line {
                None => eof = true,
                Some((_, line)) => {
                    eof = false;
                    lines.push(line)
                }
            }

            let process_is_running = self.child.check_state_nbl() == ProcessState::Running;
            let process_finished = !process_is_running;
            if process_finished && eof {
                break;
            }
        }

        let lines = lines.into_iter().map(|s| Rc::new(s)).collect();
        let output = ProcessOutput::new(
            None,
            None,
            lines,
            self.child.exit_code().unwrap(),
            Self::strategy(),
        );
        Ok(output)
    }

    fn strategy() -> OCatchStrategy {
        OCatchStrategy::StdCombined
    }
}

/// Reader for [`crate::OCatchStrategy::StdSeparately`].
/// Catches `STDOUT` and `STDERR`, but the order of
/// `"STDCOMBINED"` is only maybe correct.
// #[derive(Debug)]
pub struct SimultaneousOutputReader {
    stdout_pipe: Arc<Mutex<Pipe>>,
    stderr_pipe: Arc<Mutex<Pipe>>,
    child: Arc<Mutex<ChildProcess>>,
}

impl SimultaneousOutputReader {
    pub fn new(child: Arc<Mutex<ChildProcess>>) -> Self {
        let stdout_pipe = {
            child
                .as_ref()
                .lock()
                .as_ref()
                .unwrap()
                .stdout_pipe()
                .clone()
        };
        let stderr_pipe = {
            child
                .as_ref()
                .lock()
                .as_ref()
                .unwrap()
                .stderr_pipe()
                .clone()
        };
        SimultaneousOutputReader {
            stdout_pipe,
            stderr_pipe,
            child,
        }
    }

    /// Thread function that reads all lines either for STDERR or STDOUT. There will be two
    /// thread instances of this, if this strategy is choosen.
    fn thread_fn(
        pipe: Arc<Mutex<Pipe>>,
        child: Arc<Mutex<ChildProcess>>,
    ) -> Result<Vec<(Instant, String)>, UECOError> {
        let pipe = pipe.lock().unwrap();
        let mut lines_by_timestamp = vec![];

        let mut eof;
        loop {
            let line = pipe.read_line()?;
            match line {
                None => eof = true,
                Some((instant, line)) => {
                    eof = false;
                    lines_by_timestamp.push((instant, line))
                }
            }

            let process_is_running =
                child.lock().unwrap().check_state_nbl() == ProcessState::Running;
            let process_finished = !process_is_running;
            if process_finished && eof {
                trace!("Child finished & read EOF");
                break;
            }
        }

        Ok(lines_by_timestamp)
    }
}

impl OutputReader for SimultaneousOutputReader {
    fn read_all_bl(&mut self) -> Result<ProcessOutput, UECOError> {
        let stdout_pipe_t = self.stdout_pipe.clone();
        let stderr_pipe_t = self.stderr_pipe.clone();
        let child_t = self.child.clone();
        let stdout_t =
            thread::spawn(move || SimultaneousOutputReader::thread_fn(stdout_pipe_t, child_t));
        let child_t = self.child.clone();
        let stderr_t =
            thread::spawn(move || SimultaneousOutputReader::thread_fn(stderr_pipe_t, child_t));

        // get lines from threads with timestamps
        let stdout = stdout_t.join().unwrap()?;
        let stderr = stderr_t.join().unwrap()?;

        // transform string to Rc<String>
        let stdout = stdout
            .into_iter()
            .map(|(i, l)| (i, Rc::new(l)))
            .collect::<Vec<(Instant, Rc<String>)>>();
        let stderr = stderr
            .into_iter()
            .map(|(i, l)| (i, Rc::new(l)))
            .collect::<Vec<(Instant, Rc<String>)>>();

        // build combined lines, sorted by timestamp
        let mut combined = BTreeMap::new();
        for (instant, line) in &stdout {
            combined.insert(instant.clone(), line.clone());
        }
        for (instant, line) in &stderr {
            combined.insert(instant.clone(), line.clone());
        }

        // remove timestamp from vector
        let stdout = stdout
            .into_iter()
            .map(|(_, l)| l)
            .collect::<Vec<Rc<String>>>();
        // remove timestamp from vector
        let stderr = stderr
            .into_iter()
            .map(|(_, l)| l)
            .collect::<Vec<Rc<String>>>();
        // owned vector
        let stdcombined = combined
            .values()
            .map(|v| v.to_owned())
            .collect::<Vec<Rc<String>>>();

        Ok(ProcessOutput::new(
            Some(stdout),
            Some(stderr),
            stdcombined,
            self.child.lock().unwrap().exit_code().unwrap(),
            Self::strategy(),
        ))
    }

    /// Getter for the used strategy to obtain the output.
    fn strategy() -> OCatchStrategy {
        OCatchStrategy::StdSeparately
    }
}
