//! Childprocess related abstractions.

use crate::error::UECOError;
use crate::libc_util::{libc_ret_to_result, LibcSyscall};
use crate::exec::exec;
use std::cell::RefCell;
use crate::pipe::Pipe;
use std::rc::Rc;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProcessState {
    Ready,
    Running,
    Failed,
    FinishedSuccess,
    FinishedError(i32),
}

// #[derive(Debug)]
pub struct ChildProcess {
    executable: String,
    args: Vec<String>,
    pid: Option<libc::pid_t>,
    exit_code: Option<i32>,
    state: ProcessState,
    stdout_pipe: Rc<RefCell<Pipe>>,
    stderr_pipe: Rc<RefCell<Pipe>>,
    /// Code that should be executed in child after fork()
    /// but before exec().
    child_after_dispatch_before_exec_fn: Box<dyn FnMut() -> Result<(), UECOError>>,
    /// Code that should be executed in parent after fork()
    parent_after_dispatch_fn: Box<dyn FnMut() -> Result<(), UECOError>>
}

impl ChildProcess {
    pub fn new(executable: &str,
               args: Vec<&str>,
               child_after_dispatch_before_exec_fn: Box<dyn FnMut() -> Result<(), UECOError>>,
               parent_after_dispatch_fn: Box<dyn FnMut() -> Result<(), UECOError>>,
               stdout_pipe: Rc<RefCell<Pipe>>, stderr_pipe: Rc<RefCell<Pipe>>,
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

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
    pub fn stdout_pipe(&self) -> &Rc<RefCell<Pipe>> {
        &self.stdout_pipe
    }
    pub fn stderr_pipe(&self) -> &Rc<RefCell<Pipe>> {
        &self.stderr_pipe
    }
}
