use std::ffi::OsStr;

use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn new_tokio_command<S: AsRef<OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    apply_hidden_window(&mut command);
    command
}

pub fn new_std_command<S: AsRef<OsStr>>(program: S) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    apply_hidden_window_std(&mut command);
    command
}

#[cfg(windows)]
fn apply_hidden_window(command: &mut Command) {
    command.creation_flags(hidden_process_creation_flags());
}

#[cfg(not(windows))]
fn apply_hidden_window(_command: &mut Command) {}

#[cfg(windows)]
fn apply_hidden_window_std(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;

    command.creation_flags(hidden_process_creation_flags());
}

#[cfg(not(windows))]
fn apply_hidden_window_std(_command: &mut std::process::Command) {}

#[cfg(windows)]
fn hidden_process_creation_flags() -> u32 {
    CREATE_NO_WINDOW
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn windows_hidden_process_flags_include_create_no_window() {
        assert_eq!(hidden_process_creation_flags() & 0x08000000, 0x08000000);
    }
}
