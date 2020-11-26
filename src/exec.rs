//! Utility functions for exec.

use std::ffi::CString;
use crate::ProcessOutput;
use crate::pipe::Pipe;
use std::rc::Rc;
use crate::error::UECOError;
use crate::libc_util::{libc_ret_to_result, LibcSyscall};

/// Wrapper around `[libc::execvp]`.
/// * `executable` Path or name of executable without null (\0).
/// * `args` vector of args without null (\0). Remember that the
///          first real arg starts at index 1. index 0 is usually
///          the name of the executable. See:
///          https://unix.stackexchange.com/questions/315812/why-does-argv-include-the-program-name
fn exec(executable: &str, args: Vec<&str>) -> Result<(), UECOError> {
    // panics if the string contains a \0 (null)
    let executable = CString::new(executable).expect("Executable must not contain null!");
    let executable = executable.as_c_str();

    // Build array of null terminated C-strings array
    let args = args
        .iter()
        .map(|s| CString::new(*s).expect("Arg not contain null!"))
        .collect::<Vec<CString>>();
    // Build null terminated array with pointers null terminated c-strings
    let mut args_nl = args.iter()
        .map(|cs| cs.as_ptr())
        .collect::<Vec<* const i8>>();
    args_nl.push(std::ptr::null());


    let ret = unsafe { libc::execvp(executable.as_ptr(), args_nl.as_ptr()) };
    let res = libc_ret_to_result(ret, LibcSyscall::Execvp);

    res
}

/// Executes a program in a child process and returns the output of STDOUT and STDERR
/// line by line.
/// Be aware that this is blocking and static! So if your executable produces 1GB of
/// output text, the returned struct is 1GB in size. If the program doesn't
/// terminate, this function will neither.
///
/// This will be fine for commands like "sysctl -a" or "ls -la" on MacOS.
///
/// * `executable` Path or name of executable without null (\0). Lookup in $PATH happens automatically.
/// * `args` vector of args, each without null (\0). Remember that the
///          first real arg starts at index 1. index 0 is usually
///          the name of the executable. See:
///          https://unix.stackexchange.com/questions/315812/why-does-argv-include-the-program-name
pub fn fork_exec_and_catch(executable: &str, args: Vec<&str>) -> Result<ProcessOutput, UECOError> {
    trace!("creating stdout pipe:");
    let mut stdout_pipe = Pipe::new()?;
    trace!("creating stderr pipe:");
    let mut stderr_pipe = Pipe::new()?;

    let pid = unsafe { libc::fork() };
    // unwrap error, if pid == -1
    libc_ret_to_result(pid, LibcSyscall::Fork)?;

    trace!("forked successfully");

    if pid == 0 {
        // child process
        trace!("Hello from Child!");

        // close write ends
        stdout_pipe.mark_as_child_process()?;
        stderr_pipe.mark_as_child_process()?;

        let res = unsafe { libc::dup2(stdout_pipe.write_fd(), libc::STDOUT_FILENO) };
        // unwrap error, if res == -1
        libc_ret_to_result(res, LibcSyscall::Dup2)?;
        let res = unsafe { libc::dup2(stderr_pipe.write_fd(), libc::STDERR_FILENO) };
        // unwrap error, if res == -1
        libc_ret_to_result(res, LibcSyscall::Dup2)?;

        exec(executable, args)?;
        // here be dragons (after exec())
        // only happens if exec failed; otherwise at this point
        // the address space of the process is replaced by the new program
    } else {
        // parent process
        trace!("Hello from parent!");

        // close read ends
        stdout_pipe.mark_as_parent_process()?;
        stderr_pipe.mark_as_parent_process()?;

        // all lines from stdout of child process land here
        let mut stdout_lines = vec![];
        // all lines from stdout of child process land here
        let mut stderr_lines = vec![];
        // all lines from both streams land here in the order
        // they occured
        let mut stdcombined_lines = vec![];

        // this loop works ONLY if the program terminated.
        // so "cat /dev/random" will result in an infinite loop.
        // If so: even if our process reads faster than
        // the child process produces; due to read() our consuming process
        // gets blocked by the kernel (as long as the pipe lives and is not closed
        // by the child, like when exiting).
        loop {
            let mut stdout_eof = false;
            let mut stderr_eof = false;

            let stdout_line = stdout_pipe.read_line()?;
            let stderr_line = stderr_pipe.read_line()?;

            let stdout_line = stdout_line.map(|l| Rc::new(l));
            let stderr_line = stderr_line.map(|l| Rc::new(l));

            if let Some(l) = stdout_line {
                stdout_lines.push(l.clone());
                stdcombined_lines.push(l);
            } else {
                stdout_eof = true;
            }

            if let Some(l) = stderr_line {
                stderr_lines.push(l.clone());
                stdcombined_lines.push(l);
            } else {
                stderr_eof = true;
            }

            if stderr_eof && stdout_eof { break; }
        }

        let res = ProcessOutput::new(
            stdout_lines,
            stderr_lines,
            stdcombined_lines
        );

        return Ok(res);

    }

    Err(UECOError::Unknown)
}
