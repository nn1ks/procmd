//! A helper library for building commands.
//!
//! The [`cmd!`] macro can be used to generate [`std::process::Command`] (or [`PipeCommand`]). Refer
//! to its documentation for more information.
//!
//! # Examples
//!
//! ```rust
//! use procmd::cmd;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file_path = Path::new("/path/to/file");
//! cmd!("cat", "-n", file_path).spawn()?;
//! # Ok(())
//! # }
//! ```

#![feature(min_const_generics)]
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, missing_docs, missing_debug_implementations)]

use std::io;
use std::process::{Child, Command, ExitStatus, Output, Stdio};

/// A macro for building commands.
///
/// # Generating a simple command
///
/// To generate a [`std::process::Command`], the program and additional arguments can be passed to
/// the macro.
///
/// ## Example
///
/// The invocation:
///
/// ```rust
/// # use procmd::cmd;
/// let cmd = cmd!("ls", "-a", "/");
/// ```
///
/// expands to:
///
/// ```rust
/// let cmd = {
///     let mut cmd = ::std::process::Command::new("ls");
///     cmd.arg("-a");
///     cmd.arg("/");
///     cmd
/// };
/// ```
///
/// # Generating a piped command
///
/// To generate a [`PipeCommand`], multiple programs and arguments seperated by `=>` can be passed
/// to the macro.
///
/// [`PipeCommand`]: crate::PipeCommand
///
/// ## Example
///
/// The invocation:
///
/// ```rust
/// # use procmd::cmd;
/// let pipe_cmd = cmd!("ls" => "grep", "test" => "wc", "-l");
/// ```
///
/// expands to:
///
/// ```rust
/// # use procmd::cmd;
/// let pipe_cmd = ::procmd::PipeCommand::new([
///     {
///         let mut cmd = ::std::process::Command::new("ls");
///         cmd
///     },
///     {
///         let mut cmd = ::std::process::Command::new("grep");
///         cmd.arg("test");
///         cmd
///     },
///     {
///         let mut cmd = ::std::process::Command::new("wc");
///         cmd.arg("-l");
///         cmd
///     },
/// ]);
/// ```
pub use procmd_macro::cmd;

/// Multiple commands that will be piped.
///
/// A [`PipeCommand`] can be created by either using the [`new`] method or by using the [`cmd!`]
/// macro.
///
/// # Examples
///
/// Using the [`new`] method combined with [`Command::new`] and a simple use of the [`cmd!`] macro:
///
/// ```rust
/// use procmd::{cmd, PipeCommand};
/// use std::process::Command;
///
/// # fn main() -> Result<(), std::io::Error> {
/// let mut pipe_cmd = PipeCommand::new([Command::new("ls"), cmd!("grep", "example")]);
/// let child = pipe_cmd.spawn()?;
/// # Ok(())
/// # }
/// ```
///
/// Using the [`cmd!`] macro with the `=>` token to generate a [`PipeCommand`] and calling the
/// [`status`] method to get the exit status:
///
/// ```rust
/// use procmd::cmd;
///
/// # fn main() -> Result<(), std::io::Error> {
/// let mut pipe_cmd = cmd!("ls" => "grep", "example");
/// let exit_status = pipe_cmd.status()?;
/// # Ok(())
/// # }
/// ```
///
/// [`new`]: Self::new
/// [`status`]: Self::status
/// [`cmd!`]: crate::cmd
#[derive(Debug)]
pub struct PipeCommand<const N: usize> {
    /// The commands.
    pub commands: [Command; N],
}

impl<const N: usize> PipeCommand<N> {
    /// Creates a new [`PipeCommand`].
    pub fn new(commands: [Command; N]) -> Self {
        Self { commands }
    }

    /// Spawns all commands except the last one and calls `f` on the last command.
    ///
    /// # Panics
    ///
    /// This method panics if [`commands`] is empty.
    ///
    /// [`commands`]: Self::commands
    fn run<F, U>(&mut self, f: F) -> io::Result<U>
    where
        F: Fn(&mut Command) -> io::Result<U>,
    {
        let mut child: Option<Child> = None;
        let commands_len = self.commands.len();
        for (i, command) in self.commands[..commands_len - 1].iter_mut().enumerate() {
            if let Some(child) = child {
                command.stdin(child.stdout.unwrap());
            }
            if i == commands_len - 1 {
                break;
            } else {
                command.stdout(Stdio::piped());
                child = Some(command.spawn()?);
            }
        }
        f(&mut self.commands[commands_len - 1])
    }

    /// Spawns all commands and returns the [`Child`] of the last command.
    ///
    /// # Panics
    ///
    /// This method panics if [`commands`] is empty.
    ///
    /// [`commands`]: Self::commands
    pub fn spawn(&mut self) -> io::Result<Child> {
        self.run(|command| command.spawn())
    }

    /// Returns the [`Output`] of the last command.
    ///
    /// Note that this method still calls [`Command::spawn`] on all commands except the last one.
    ///
    /// # Panics
    ///
    /// This method panics if [`commands`] is empty.
    ///
    /// [`commands`]: Self::commands
    pub fn output(&mut self) -> io::Result<Output> {
        self.run(|command| command.output())
    }

    /// Returns the [`ExitStatus`] of the last command.
    ///
    /// Note that this method still calls [`Command::spawn`] on all commands except the last one.
    ///
    /// # Panics
    ///
    /// This method panics if [`commands`] is empty.
    ///
    /// [`commands`]: Self::commands
    pub fn status(&mut self) -> io::Result<ExitStatus> {
        self.run(|command| command.status())
    }
}
