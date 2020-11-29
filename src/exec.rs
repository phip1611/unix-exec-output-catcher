//! Utility functions for exec.

use std::ffi::CString;
use crate::ProcessOutput;
use crate::pipe::Pipe;
use std::rc::Rc;
use crate::error::UECOError;
use crate::libc_util::{libc_ret_to_result, LibcSyscall};
use crate::child::{ChildProcess, ProcessState};
use std::cell::RefCell;

/// Wrapper around `[libc::execvp]`.
/// * `executable` Path or name of executable without null (\0).
/// * `args` vector of args without null (\0). Remember that the
///          first real arg starts at index 1. index 0 is usually
///          the name of the executable. See:
///          https://unix.stackexchange.com/questions/315812/why-does-argv-include-the-program-name
pub fn exec(executable: &str, args: Vec<&str>) -> Result<(), UECOError> {
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
    let mut stdout_pipe = Rc::new(RefCell::new(Pipe::new()?));
    let mut stderr_pipe = Rc::new(RefCell::new(Pipe::new()?));

    let stdout_pipe_cap = stdout_pipe.clone();
    let stderr_pipe_cap = stderr_pipe.clone();
    let child_after_dispatch_before_exec_fn = move || {
        child_setup_pipes(stdout_pipe_cap.clone(), stderr_pipe_cap.clone())
        // Err(UECOError::Unknown)
    };
    let child_after_dispatch_before_exec_fn = Box::new(child_after_dispatch_before_exec_fn);
    let stdout_pipe_cap = stdout_pipe.clone();
    let stderr_pipe_cap = stderr_pipe.clone();
    let parent_after_dispatch_fn = move || {
        parent_setup_pipes(stdout_pipe_cap.clone(), stderr_pipe_cap.clone())
        // Err(UECOError::Unknown)
    };
    let parent_after_dispatch_fn = Box::new(parent_after_dispatch_fn);
    let mut child = ChildProcess::new(
        executable,
        args,
        child_after_dispatch_before_exec_fn,
        parent_after_dispatch_fn,
    );

    child.dispatch()?;
    // call this after dispatch
    let res = parent_catch_output(stdout_pipe.clone(), stderr_pipe.clone(), &mut child)?;

    Ok(res)
}

fn child_setup_pipes(stdout_pipe: Rc<RefCell<Pipe>>, stderr_pipe: Rc<RefCell<Pipe>>) -> Result<(), UECOError> {
    let mut stdout_pipe = stdout_pipe.borrow_mut();
    let mut stderr_pipe = stderr_pipe.borrow_mut();

    trace!("After fork child setup code called!");

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

/// CALL THIS AFTER CHILD PROCESS WAS DISPATCHED!
fn parent_setup_pipes(stdout_pipe: Rc<RefCell<Pipe>>, stderr_pipe: Rc<RefCell<Pipe>>) -> Result<(), UECOError> {
    let mut stdout_pipe = stdout_pipe.borrow_mut();
    let mut stderr_pipe = stderr_pipe.borrow_mut();


    trace!("After fork parent setup code called!");

    // close read ends
    stdout_pipe.mark_as_parent_process()?;
    stderr_pipe.mark_as_parent_process()?;
    Ok(())
}

fn parent_catch_output(stdout_pipe: Rc<RefCell<Pipe>>, stderr_pipe: Rc<RefCell<Pipe>>, child: &mut ChildProcess) -> Result<ProcessOutput, UECOError> {
    let mut stdout_pipe = stdout_pipe.borrow_mut();
    let mut stderr_pipe = stderr_pipe.borrow_mut();

    // all lines from stdout of child process land here
    let mut stdout_lines = vec![];
    // all lines from stdout of child process land here
    let mut stderr_lines = vec![];
    // all lines from both streams land here in the order
    // they occured
    let mut stdcombined_lines = vec![];

    // this loop works ONLY if the program terminated.
    // so "cat /dev/random" will result in an infinite loop.


    let mut stdout_eof = false;
    let mut stderr_eof = false;
    loop {
        let process_is_running = child.check_state_nbl() == ProcessState::Running;
        let process_finished = !process_is_running;
        if process_finished && stdout_eof && stderr_eof {
            trace!("Child finished and stdout and stderr signal EOF");
            break;
        }

        let stdout_line = stdout_pipe.read_line()?;
        let stderr_line = stderr_pipe.read_line()?;

        let stdout_line = stdout_line.map(|l| Rc::new(l));
        let stderr_line = stderr_line.map(|l| Rc::new(l));

        if let Some(l) = stdout_line {
            stdout_lines.push(l.clone());
            stdcombined_lines.push(l);
            stdout_eof = true;
        } else {
            stdout_eof = true;
        }

        if let Some(l) = stderr_line {
            stderr_lines.push(l.clone());
            stdcombined_lines.push(l);
            stderr_eof = false;
        } else {
            stderr_eof = true;
        }
    }
    trace!("Excited read loop!");

    let res = ProcessOutput::new(
        stdout_lines,
        stderr_lines,
        stdcombined_lines,
        // exit_code
        0 // TODO
    );

    Ok(res)
}


