//! Contains all errors that can happen in this library.

use derive_more::Display;
use std::error::Error;

/// Short for U(nix) E(xec) C(atch) O(utput)-Error.
/// Combines all errors that can happen inside this library.
#[derive(Debug, Display, Copy, Clone)]
pub enum UECOError {
    #[display(fmt = "pipe() failed with error code {}", errno)]
    PipeFailed{errno: i32},
    #[display(fmt = "dup2() failed with error code {}", errno)]
    Dup2Failed{errno: i32},
    #[display(fmt = "execvp() failed with error code {}", errno)]
    ExecvpFailed{errno: i32},
    #[display(fmt = "waitpid() failed with error code {}", errno)]
    WaitpidFailed{errno: i32},
    #[display(fmt = "read() failed with error code {}", errno)]
    ReadFailed{errno: i32},
    #[display(fmt = "fork() failed with error code {}", errno)]
    ForkFailed{errno: i32},
    #[display(fmt = "close() failed with error code {}", errno)]
    CloseFailed{errno: i32},
    #[display(fmt = "The pipe is not yet marked as read end.")]
    PipeNotMarkedAsReadEnd,
    #[display(fmt = "The child was already dispatched/started.")]
    ChildAlreadyDispatched,


    /// For all other errors.
    Unknown,
}

// IDE might show that display is not implemented but it gets implemented
// during build by "derive_more" crate
impl Error for UECOError {}
