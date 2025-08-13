/*!
Run a command in a fake tty
Copied from <https://github.com/dtolnay/faketty/>, which unfortunately doesn't offer a library (see <https://github.com/dtolnay/faketty/issues/10>).
*/

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(
    clippy::empty_enum,
    clippy::indexing_slicing,
    clippy::let_underscore_untyped,
    clippy::missing_docs_in_private_items,
    clippy::needless_pass_by_value,
    clippy::undocumented_unsafe_blocks,
    clippy::uninlined_format_args
)]

use crate::opts::FakettyOptions;
use color_eyre::Result;
use nix::pty;
use nix::pty::ForkptyResult;
use nix::pty::Winsize;
use nix::sys::wait;
use nix::sys::wait::WaitStatus;
use nix::unistd;
use nix::unistd::Pid;
use std::ffi::CString;
use std::os::fd::AsFd;
use std::os::fd::BorrowedFd;
use std::os::unix::ffi::OsStrExt;
use std::process;

enum Exec {}

const STDIN: BorrowedFd = unsafe { BorrowedFd::borrow_raw(0) };
const STDOUT: BorrowedFd = unsafe { BorrowedFd::borrow_raw(1) };
const STDERR: BorrowedFd = unsafe { BorrowedFd::borrow_raw(2) };

pub(crate) fn run(faketty_options: FakettyOptions) -> Result<()> {
    let args: Vec<CString> = faketty_options
        .program
        .into_iter()
        .map(|os_string| CString::new(os_string.as_bytes()))
        .collect::<Result<_, _>>()?;

    let new_stdin = STDIN.try_clone_to_owned()?;
    let new_stderr = STDERR.try_clone_to_owned()?;
    let pty1 = unsafe { forkpty() }?;
    if let ForkptyResult::Parent { child, master } = pty1 {
        copyfd(master.as_fd(), STDOUT);
        copyexit(child);
    }
    let new_stdout = STDOUT.try_clone_to_owned()?;
    let pty2 = unsafe { forkpty() }?;
    if let ForkptyResult::Parent { child, master } = pty2 {
        copyfd(master.as_fd(), new_stderr.as_fd());
        copyexit(child);
    }
    unistd::dup2_stdin(new_stdin)?;
    unistd::dup2_stdout(new_stdout)?;
    exec(args).map(|_| ())
}

unsafe fn forkpty() -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    let result = unsafe { pty::forkpty(&winsize, termios) }?;
    Ok(result)
}

fn exec(args: Vec<CString>) -> Result<Exec> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

fn copyfd(read: BorrowedFd, write: BorrowedFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read, &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: BorrowedFd, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd, buf)?;
        buf = &buf[n..];
    }
    Ok(())
}

fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}
