mod raw_guard;

#[cfg(feature = "backend-tokio")]
mod main {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    pub async fn run(
        child: &mut pty_process::tokio::Child,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _raw = super::raw_guard::RawGuard::new();

        let mut in_buf = [0_u8; 4096];
        let mut out_buf = [0_u8; 4096];

        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();

        loop {
            tokio::select! {
                bytes = stdin.read(&mut in_buf) => match bytes {
                    Ok(bytes) => {
                        child.pty_mut().write_all(&in_buf[..bytes]).await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("stdin read failed: {:?}", e);
                        break;
                    }
                },
                bytes = child.pty_mut().read(&mut out_buf) => match bytes {
                    Ok(bytes) => {
                        stdout.write_all(&out_buf[..bytes]).await.unwrap();
                        stdout.flush().await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("pty read failed: {:?}", e);
                        break;
                    }
                },
                // _ = child.status_no_drop() => {
                //     break;
                // }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "backend-tokio")]
#[tokio::main]
async fn main() {
    use std::os::unix::process::ExitStatusExt as _;

    let pty = pty_process::tokio::Pty::new().unwrap();
    pty.resize(pty_process::Size::new(24, 80)).unwrap();
    let mut cmd = pty_process::tokio::Command::new("tac");
    // cmd.args(&["500"]);
    let mut child = cmd.spawn(pty).unwrap();
    main::run(&mut child).await.unwrap();
    let status = child.wait().await.unwrap();
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}

#[cfg(not(feature = "backend-tokio"))]
fn main() {
    unimplemented!()
}
