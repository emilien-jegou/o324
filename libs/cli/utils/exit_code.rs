use core::fmt::Debug;
use std::process::Termination;

/// Represents standard Unix exit codes as defined in `<sysexits.h>`.
///
/// This enum provides a type-safe way to handle program termination.
/// It implements the `std::process::Termination` trait, allowing you to
/// return a variant directly from your `main` function.
///
/// # Example
///
/// ```rust
/// fn main() -> ExitCode {
///     let args: Vec<String> = std::env::args().collect();
///
///     if args.len() != 2 {
///         eprintln!("Usage: {} <filename>", args[0]);
///         return ExitCode::Usage;
///     }
///
///     let filename = &args[1];
///     if !std::path::Path::new(filename).exists() {
///         eprintln!("Error: Input file '{}' not found.", filename);
///         return ExitCode::NoInput;
///     }
///
///     // ... program logic ...
///
///     println!("Program completed successfully.");
///     ExitCode::Success
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// The program executed successfully. (EX_OK)
    Success = 0,

    /// A generic or unspecified error occurred. (Not in sysexits.h, but common practice)
    Error = 1,

    /// The command was used incorrectly, e.g., with the wrong number of arguments,
    /// a bad flag, or a bad syntax. (EX_USAGE)
    Usage = 64,

    /// The input data was incorrect in some way. This should be used for
    /// user-supplied data, not system files. (EX_DATAERR)
    DataError = 65,

    /// An input file (not a system file) did not exist or was not readable. (EX_NOINPUT)
    NoInput = 66,

    /// A specified user did not exist. This might be used for mail handlers
    /// or other programs that look up users. (EX_NOUSER)
    NoUser = 67,

    /// A specified host did not exist. (EX_NOHOST)
    NoHost = 68,

    /// A required service is unavailable. This can be used for network
    /// connectivity problems, etc. (EX_UNAVAILABLE)
    Unavailable = 69,

    /// An internal software error has been detected. This should be limited to
    /// non-recoverable errors. (EX_SOFTWARE)
    Software = 70,

    /// An operating system error has been detected. This is intended for errors
    /// like "cannot fork" or "cannot create pipe". (EX_OSERR)
    OsError = 71,

    /// Some system file (e.g., /etc/passwd) does not exist, cannot be opened,
    /// or has some other kind of error. (EX_OSFILE)
    OsFile = 72,

    /// A user-specified output file cannot be created. (EX_CANTCREAT)
    CantCreate = 73,

    /// An error occurred while doing I/O on some file. (EX_IOERR)
    IoError = 74,

    /// A temporary failure occurred. The user is invited to try again later.
    /// (e.g., a network connection that couldn't be established). (EX_TEMPFAIL)
    TempFail = 75,

    /// A protocol error was detected during a remote transaction. (EX_PROTOCOL)
    Protocol = 76,

    /// The user did not have sufficient permissions to perform the operation. (EX_NOPERM)
    PermissionDenied = 77,

    /// A configuration error was detected. (EX_CONFIG)
    ConfigError = 78,
}

impl ExitCode {
    /// Terminates the current process with the corresponding exit code.
    ///
    /// This function will never return.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn do_something_critical() {
    ///     // ...
    ///     if something_bad_happened {
    ///         eprintln!("A critical error occurred!");
    ///         ExitCode::Software.exit();
    ///     }
    /// }
    /// ```
    pub fn exit(self) -> ! {
        std::process::exit(self as i32)
    }

    /// Returns the integer value of the exit code.
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// Allows `ExitCode` to be returned from `main`.
impl Termination for ExitCode {
    fn report(self) -> std::process::ExitCode {
        self.code().into()
    }
}
