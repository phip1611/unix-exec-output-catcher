//! Childprocess related abstractions.

use crate::error::UECOError;
use crate::libc_util::{libc_ret_to_result, LibcSyscall};
use crate::exec::exec;
use crate::pipe::Pipe;
use std::sync::{Mutex, Arc};
use std::fmt::Debug;

/// The state in that a child process can be.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProcessState {
    /// Waiting for dispatch.
    Ready,
    /// Dispatched.
    Running,
    /// Finished with error code 0.
    FinishedSuccess,
    /// Finished with error code != 0.
    FinishedError(i32),
}

/// Abstraction over a child process.
pub struct ChildProcess {
    /// String of the executable. Can also be a name
    /// that will be looked up inside $PATH during execution.
    executable: String,
    /// All args of the program including args[0] that refers to
    /// the name of the binary.
    args: Vec<String>,
    /// Once the process has been dispatched/forked, the pid of the child
    /// is set here.
    pid: Option<libc::pid_t>,
    /// Once the process exited, the exit code stands here.
    exit_code: Option<i32>,
    /// The current process state.
    state: ProcessState,
    /// Reference to the pipe where STDOUT gets redirected.
    stdout_pipe: Arc<Mutex<Pipe>>,
    /// Reference to the pipe where STDERR gets redirected.
    stderr_pipe: Arc<Mutex<Pipe>>,
    /// Code that should be executed in child after fork() but before exec().
    child_after_dispatch_before_exec_fn: Box<dyn Send + FnMut() -> Result<(), UECOError>>,
    /// Code that should be executed in parent after fork()
    parent_after_dispatch_fn: Box<dyn Send + FnMut() -> Result<(), UECOError>>
}

impl ChildProcess {
    /// Constructor.
    /// * `executable` executable or path to executable
    /// * `args` Args vector. First real arg starts at index 1.
    /// * `child_after_dispatch_before_exec_fn` Code that should be executed in child after fork() but before exec().
    /// * `parent_after_dispatch_fn` Code that should be executed in parent after fork()
    /// * `stdout_pipe` Reference to the pipe where STDOUT gets redirected.
    /// * `stderr_pipe` Reference to the pipe where STDERR gets redirected.
    pub fn new(executable: &str,
               args: Vec<&str>,
               child_after_dispatch_before_exec_fn: Box<dyn Send + FnMut() -> Result<(), UECOError>>,
               parent_after_dispatch_fn: Box<dyn Send + FnMut() -> Result<(), UECOError>>,
               stdout_pipe: Arc<Mutex<Pipe>>,
               stderr_pipe: Arc<Mutex<Pipe>>,
    ) -> Self {
        ChildProcess {
            executable: executable.to_string(),
            args: args.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
            pid: None,
            exit_code: None,
            state: ProcessState::Ready,
            child_after_dispatch_before_exec_fn,
            parent_after_dispatch_fn,
            stdout_pipe,
            stderr_pipe,
        }
    }

    /// Forks the process. This mean child and parent will run from that
    /// point concurrently.
    pub fn dispatch(&mut self) -> Result<libc::pid_t, UECOError> {
        self.state = ProcessState::Running;
        let pid = unsafe { libc::fork() };
        // unwrap error, if pid == -1
        libc_ret_to_result(pid, LibcSyscall::Fork)?;

        trace!("forked successfully");

        if pid == 0 {
            // child process
            trace!("Hello from Child!");
            let res: Result<(), UECOError> = (self.child_after_dispatch_before_exec_fn)();
            res?;
            exec(&self.executable, self.args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
            // here be dragons (after exec())
            // only happens if exec failed; otherwise at this point
            // the address space of the process is replaced by the new program
            Err(UECOError::Unknown)
        } else {
            // parent process
            trace!("Hello from parent!");
            self.pid.replace(pid);
            let res: Result<(), UECOError> = (self.parent_after_dispatch_fn)();
            res?;
            Ok(pid)
        }
    }

    /// Check process state nonblocking from parent.
    pub fn check_state_nbl(&mut self) -> ProcessState {
        if self.state != ProcessState::Running {
            return self.state;
        }

        let wait_flags = libc::WNOHANG;
        let mut status_code: libc::c_int = 0;
        let status_code_ptr = &mut status_code as * mut libc::c_int;

        let ret = unsafe { libc::waitpid(self.pid.unwrap(), status_code_ptr, wait_flags) };
        libc_ret_to_result(ret, LibcSyscall::Waitpid).unwrap();

        // IDE doesn't find this functions but they exist

        // I'm not sure on this one..
        // maybe my assumption is wrong and "child process started"
        // is actually printed when the process is finished.
        // But this is not important for the problem here.

        // process didn't started running yet
        if ret == 0 {
            trace!("Child process not started yet");
            return self.state; // RUNNING
        } else if ret == self.pid.unwrap() {
            trace!("Child process started");
        }

        // returns true if the child terminated normally
        let exited_normally: bool = libc::WIFEXITED(status_code);
        // returns true if the child was terminated by signal
        let exited_by_signal: bool = libc::WIFSIGNALED(status_code);
        // exit code (0 = success, or > 1 = error)
        let exit_code: libc::c_int = libc::WEXITSTATUS(status_code);


        if exited_normally || exited_by_signal {
            self.exit_code.replace(exit_code);
            if exit_code == 0 {
                self.state = ProcessState::FinishedSuccess;
            } else {
                self.state = ProcessState::FinishedError(exit_code);
            }
        }

        self.state
    }

    /// Getter for exit code.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
    /// Getter for stdout_pipe.
    pub fn stdout_pipe(&self) -> &Arc<Mutex<Pipe>> {
        &self.stdout_pipe
    }
    /// Getter for stderr_pipe.
    pub fn stderr_pipe(&self) -> &Arc<Mutex<Pipe>> {
        &self.stderr_pipe
    }
}
