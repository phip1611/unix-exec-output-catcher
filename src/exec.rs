//! Utility functions for exec.

use std::ffi::CString;
use crate::ProcessOutput;
use crate::pipe::{CatchPipes};
use crate::error::UECOError;
use crate::libc_util::{libc_ret_to_result, LibcSyscall};
use crate::child::{ChildProcess};
use crate::OCatchStrategy;
use crate::reader::{OutputReader, SimpleOutputReader, SimultaneousOutputReader};
use std::sync::{Arc, Mutex};

/// Wrapper around [`libc::execvp`].
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
/// line by line in a vector. Be aware that this is blocking and static! So if your
/// executable produces 1GB of output text, the data of the vectors of the returned structs
/// is also 1GB in size. If the program doesn't terminate, this function will neither.
///
/// This lib/function works fine for commands like "$ sysctl -a" or "$ ls -la".
///
/// ⚠️ Difference to std::process::Command 🚨
/// `std::process::Command` does the same in the standard library but **with one exception**:
/// My library gives you access to stdout, stderr, **and "stdcombined"**. This way you get all output
/// lines in the order they appeared. That's the unique feature of this crate.
///
///
/// * `executable` Path or name of executable without null (\0). Lookup in $PATH happens automatically.
/// * `args` vector of args, each without null (\0). Remember that the
///          first real arg starts at index 1. index 0 is usually
///          the name of the executable. See:
///          https://unix.stackexchange.com/questions/315812/why-does-argv-include-the-program-name
/// * `strategy` Specify how accurate the `"STDCOMBINED` vecor is. See [`crate::OCatchStrategy`] for
///              more information.
pub fn fork_exec_and_catch(executable: &str, args: Vec<&str>, strategy: OCatchStrategy) -> Result<ProcessOutput, UECOError> {
    let cp = CatchPipes::new(strategy)?;
    let child = match strategy {
        OCatchStrategy::StdCombined => { setup_and_execute_strategy_combined(executable, args, cp) }
        OCatchStrategy::StdSeparately => { setup_and_execute_strategy_separately(executable, args, cp) }
    };
    let mut child = child?;
    child.dispatch()?;
    let output = match strategy {
        OCatchStrategy::StdCombined => { SimpleOutputReader::new(&mut child).read_all_bl() }
        OCatchStrategy::StdSeparately => { SimultaneousOutputReader::new(Arc::new(Mutex::new(child))).read_all_bl() }
    };
    output
}

/// Setups up parent and child process and executes everything. Obtains the output
/// using the [`crate::OCatchStrategy::StdCombined`]-strategy.
fn setup_and_execute_strategy_combined(executable: &str, args: Vec<&str>, cp: CatchPipes) -> Result<ChildProcess, UECOError> {
    let pipe = if let CatchPipes::Combined(pipe) = cp { pipe } else { panic!("Wrong CatchPipe-variant") };
    let pipe = Arc::new(Mutex::new(pipe));
    let pipe_closure = pipe.clone();
    // gets called after fork() after
    let child_setup = move || {
        let mut pipe_closure = pipe_closure.lock().unwrap();
        pipe_closure.mark_as_child_process()?;
        pipe_closure.connect_to_stdout()?;
        pipe_closure.connect_to_stderr()?;
        Ok(())
    };
    let pipe_closure = pipe.clone();
    let parent_setup = move || {
        let mut pipe_closure = pipe_closure.lock().unwrap();
        pipe_closure.mark_as_parent_process()?;
        Ok(())
    };
    let child = ChildProcess::new(
        executable,
        args,
        Box::new(child_setup),
        Box::new(parent_setup),
        pipe.clone(),
        pipe,
    );
    Ok(child)
}

/// Setups up parent and child process and executes everything. Obtains the output
/// using the [`crate::OCatchStrategy::StdSeparately`]-strategy.
fn setup_and_execute_strategy_separately(executable: &str, args: Vec<&str>, cp: CatchPipes) -> Result<ChildProcess, UECOError> {
    let (stdout_pipe, stderr_pipe) = if let CatchPipes::Separately{stdout, stderr} = cp {
        (stdout, stderr)
    } else { panic!("Wrong CatchPipe-variant") };
    let stdout_pipe = Arc::new(Mutex::new(stdout_pipe));
    let stderr_pipe = Arc::new(Mutex::new(stderr_pipe));
    let stdout_pipe_closure = stdout_pipe.clone();
    let stderr_pipe_closure = stderr_pipe.clone();
    // gets called after fork() after
    let child_setup = move || {
        let mut stdout_pipe_closure = stdout_pipe_closure.lock().unwrap();
        let mut stderr_pipe_closure = stderr_pipe_closure.lock().unwrap();
        stdout_pipe_closure.mark_as_child_process()?;
        stderr_pipe_closure.mark_as_child_process()?;
        stdout_pipe_closure.connect_to_stdout()?;
        stderr_pipe_closure.connect_to_stderr()?;
        Ok(())
    };
    let stdout_pipe_closure = stdout_pipe.clone();
    let stderr_pipe_closure = stderr_pipe.clone();
    let parent_setup = move || {
        let mut stdout_pipe_closure = stdout_pipe_closure.lock().unwrap();
        let mut stderr_pipe_closure = stderr_pipe_closure.lock().unwrap();
        stdout_pipe_closure.mark_as_parent_process()?;
        stderr_pipe_closure.mark_as_parent_process()?;
        Ok(())
    };
    let child = ChildProcess::new(
        executable,
        args,
        Box::new(child_setup),
        Box::new(parent_setup),
        stdout_pipe,
        stderr_pipe,
    );
    Ok(child)
}


