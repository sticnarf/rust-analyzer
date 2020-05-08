//! This crate contains a single public function
//! [`get_path_for_executable`](fn.get_path_for_executable.html).
//! See docs there for more information.

use anyhow::{bail, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Return a `PathBuf` to use for the given executable.
///
/// E.g., `get_path_for_executable("cargo")` may return just `cargo` if that
/// gives a valid Cargo executable; or it may return a full path to a valid
/// Cargo.
pub fn get_path_for_executable(executable_name: impl AsRef<str>) -> Result<PathBuf> {
    // The current implementation checks three places for an executable to use:
    // 1) Appropriate environment variable (erroring if this is set but not a usable executable)
    //      example: for cargo, this checks $CARGO environment variable; for rustc, $RUSTC; etc
    // 2) `<executable_name>`
    //      example: for cargo, this tries just `cargo`, which will succeed if `cargo` is on the $PATH
    // 3) `~/.cargo/bin/<executable_name>`
    //      example: for cargo, this tries ~/.cargo/bin/cargo
    //      It seems that this is a reasonable place to try for cargo, rustc, and rustup
    let executable_name = executable_name.as_ref();
    let env_var = executable_name.to_ascii_uppercase();
    if let Ok(path) = env::var(&env_var) {
        if is_valid_executable(&path) {
            Ok(path.into())
        } else {
            bail!(
                "`{}` environment variable points to something that's not a valid executable",
                env_var
            )
        }
    } else {
        if is_valid_executable(executable_name) {
            return Ok(executable_name.into());
        }
        if let Some(mut path) = ::home::home_dir() {
            path.push(".cargo");
            path.push("bin");
            path.push(executable_name);
            if is_valid_executable(&path) {
                return Ok(path);
            }
        }
        // This error message may also be caused by $PATH or $CARGO/$RUSTC/etc not being set correctly
        // for VSCode, even if they are set correctly in a terminal.
        // On macOS in particular, launching VSCode from terminal with `code <dirname>` causes VSCode
        // to inherit environment variables including $PATH, $CARGO, $RUSTC, etc from that terminal;
        // but launching VSCode from Dock does not inherit environment variables from a terminal.
        // For more discussion, see #3118.
        bail!(
            "Failed to find `{}` executable. Make sure `{}` is in `$PATH`, or set `${}` to point to a valid executable.",
            executable_name, executable_name, env_var
        )
    }
}

/// Does the given `Path` point to a usable executable?
///
/// (assumes the executable takes a `--version` switch and writes to stdout,
/// which is true for `cargo`, `rustc`, and `rustup`)
fn is_valid_executable(p: impl AsRef<Path>) -> bool {
    Command::new(p.as_ref()).arg("--version").output().is_ok()
}