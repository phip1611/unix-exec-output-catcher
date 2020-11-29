//! libc utilities

use crate::error::UECOError;

pub enum LibcSyscall {
    Fork,
    Pipe,
    Dup2,
    Close,
    Read,
    Execvp,
    Waitpid
}

pub fn libc_ret_to_result(res: libc::c_int, syscall: LibcSyscall) -> Result<(), UECOError> {
    if res == 0 {
        Ok(())
    } else if res == -1 {
        let errno = errno::errno().0;
        let err = syscall_to_uecoerror(syscall, errno);
        Err(err)
    } else {
        // also okay, because for example fork() returns the pid
        // and > 0 is a valid value :)
        Ok(())
    }
}

fn syscall_to_uecoerror(syscall: LibcSyscall, errno: libc::c_int) -> UECOError {
    match syscall {
        LibcSyscall::Fork => { UECOError::ForkFailed {errno} }
        LibcSyscall::Pipe => { UECOError::PipeFailed {errno} }
        LibcSyscall::Dup2 => { UECOError::Dup2Failed {errno} }
        LibcSyscall::Close => { UECOError::CloseFailed {errno} }
        LibcSyscall::Read => { UECOError::ReadFailed {errno} }
        LibcSyscall::Execvp => { UECOError::ExecvpFailed {errno} }
        LibcSyscall::Waitpid => { UECOError::WaitpidFailed {errno} }
    }
}
