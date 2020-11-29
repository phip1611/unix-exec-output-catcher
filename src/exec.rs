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
/// line by line in a vector.
/// Be aware that this is blocking and static! So if your executable produces 1GB of
/// output text, the data of the vectors of the returned structs are 1GB in size.
///
/// If the program doesn't terminate, this function will neither.
///
/// This will be fine for commands like "sysctl -a" or "ls -la" on MacOS.
///
/// ⚠️ Difference to std::process::Command 🚨
/// `std::process::Command` does the same in the standard library but **with one exception**:
/// My library gives you access to stdout, stderr, **and "stdcombined"**. This way you get all output
/// lines in the order they appeared. That's the unique feature of this crate.
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

        child_setup_pipes(&mut stdout_pipe, &mut stderr_pipe)?;

        exec(executable, args)?;
        // here be dragons (after exec())
        // only happens if exec failed; otherwise at this point
        // the address space of the process is replaced by the new program
    } else {
        // parent process
        trace!("Hello from parent!");

        parent_setup_pipes(&mut stdout_pipe, &mut stderr_pipe)?;
        let res = parent_catch_output(&mut stdout_pipe, &mut stderr_pipe, pid)?;

        return Ok(res);
    }

    Err(UECOError::Unknown)
}

fn child_setup_pipes(stdout_pipe: &mut Pipe, stderr_pipe: &mut Pipe) -> Result<(), UECOError> {
    // close write ends
    stdout_pipe.mark_as_child_process()?;
    stderr_pipe.mark_as_child_process()?;

    let res = unsafe { libc::dup2(stdout_pipe.write_fd(), libc::STDOUT_FILENO) };
    // unwrap error, if res == -1
    libc_ret_to_result(res, LibcSyscall::Dup2)?;
    let res = unsafe { libc::dup2(stderr_pipe.write_fd(), libc::STDERR_FILENO) };
    // unwrap error, if res == -1
    libc_ret_to_result(res, LibcSyscall::Dup2)?;
    Ok(())
}

fn parent_setup_pipes(stdout_pipe: &mut Pipe, stderr_pipe: &mut Pipe) -> Result<(), UECOError> {
    // close read ends
    stdout_pipe.mark_as_parent_process()?;
    stderr_pipe.mark_as_parent_process()?;
    Ok(())
}

fn parent_catch_output(stdout_pipe: &mut Pipe, stderr_pipe: &mut Pipe, pid: libc::pid_t) -> Result<ProcessOutput, UECOError> {
    // all lines from stdout of child process land here
    let mut stdout_lines = vec![];
    // all lines from stdout of child process land here
    let mut stderr_lines = vec![];
    // all lines from both streams land here in the order
    // they occured
    let mut stdcombined_lines = vec![];

    let exit_code;

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


        let (finished, code) = child_process_done(pid);
        if finished && stdout_eof && stderr_eof {
            exit_code = code;
            break;
        }
    }

    let res = ProcessOutput::new(
        stdout_lines,
        stderr_lines,
        stdcombined_lines,
        exit_code,
    );

    Ok(res)
}

fn child_process_done(pid: libc::pid_t) -> (bool, i32) {
    let wait_flags = libc::WNOHANG;
    let mut status_code: libc::c_int = 0;
    let status_code_ptr = &mut status_code as * mut libc::c_int;

    let _ret = unsafe { libc::waitpid(pid, status_code_ptr, wait_flags) };

    // IDE doesn't find this functions but they exist

    // returns true if the child terminated normally
    let exited_normally: bool = libc::WIFEXITED(status_code);
    // returns true if the child was terminated by signal
    let exited_by_signal: bool = libc::WIFSIGNALED(status_code);
    // exit code (0 = success, or > 1 = error)
    let exit_code: libc::c_int = libc::WEXITSTATUS(status_code);


    if exited_normally || exited_by_signal {
        (true, exit_code)
    } else {
        (false, 0)
    }
}
