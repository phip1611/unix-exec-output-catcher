//! Contains all errors that can happen in this library.

use derive_more::Display;

/// Short for U(nix) E(xec) C(atch) O(utput)-Error.
/// Combines all errors that can happen inside this library.
#[derive(Debug, Display)]
pub enum UECOError {
    #[display(fmt = "pipe() failed with error code {}", errno)]
    PipeFailed{errno: i32},
    #[display(fmt = "dup2() failed with error code {}", errno)]
    Dup2Failed{errno: i32},
    #[display(fmt = "execvp() failed with error code {}", errno)]
    ExecvpFailed{errno: i32},
    #[display(fmt = "read() failed with error code {}", errno)]
    ReadFailed{errno: i32},
    #[display(fmt = "fork() failed with error code {}", errno)]
    ForkFailed{errno: i32},
    #[display(fmt = "close() failed with error code {}", errno)]
    CloseFailed{errno: i32},
    #[display(fmt = "The pipe is not yet marked as read end.")]
    PipeNotMarkedAsReadEnd,


    // for all others
    Unknown,
}