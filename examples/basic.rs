use std::io::{Read as _, Write as _};
use std::os::unix::io::AsRawFd as _;

use pty_process::Command as _;

struct RawGuard {
    termios: nix::sys::termios::Termios,
}

impl RawGuard {
    fn new() -> Self {
        let stdin = std::io::stdin().as_raw_fd();
        let termios = nix::sys::termios::tcgetattr(stdin).unwrap();
        let mut termios_raw = termios.clone();
        nix::sys::termios::cfmakeraw(&mut termios_raw);
        nix::sys::termios::tcsetattr(
            stdin,
            nix::sys::termios::SetArg::TCSANOW,
            &termios_raw,
        )
        .unwrap();
        Self { termios }
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let stdin = std::io::stdin().as_raw_fd();
        let _ = nix::sys::termios::tcsetattr(
            stdin,
            nix::sys::termios::SetArg::TCSANOW,
            &self.termios,
        );
    }
}

fn main() {
    let mut child = std::process::Command::new("sh")
        .spawn_pty(Some(&pty_process::Size::new(24, 80)))
        .unwrap();
    let _raw = RawGuard::new();
    let mut buf = [0_u8; 4096];
    let pty = child.pty().as_raw_fd();
    let stdin = std::io::stdin().as_raw_fd();
    loop {
        let mut set = nix::sys::select::FdSet::new();
        set.insert(pty);
        set.insert(stdin);
        match nix::sys::select::select(None, Some(&mut set), None, None, None)
        {
            Ok(n) => {
                if n > 0 {
                    if set.contains(pty) {
                        match child.pty().read(&mut buf) {
                            Ok(bytes) => {
                                let buf = &buf[..bytes];
                                let stdout = std::io::stdout();
                                let mut stdout = stdout.lock();
                                stdout.write_all(buf).unwrap();
                                stdout.flush().unwrap();
                            }
                            Err(e) => {
                                eprintln!("pty read failed: {:?}", e);
                                break;
                            }
                        };
                    }
                    if set.contains(stdin) {
                        match std::io::stdin().read(&mut buf) {
                            Ok(bytes) => {
                                let buf = &buf[..bytes];
                                child.pty().write_all(buf).unwrap();
                            }
                            Err(e) => {
                                eprintln!("stdin read failed: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => println!("select failed: {:?}", e),
        }
    }
    child.wait().unwrap();
}
