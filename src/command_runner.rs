use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;

pub struct CommandRunner {
    file: PathBuf,
    cmd: String,
}

impl CommandRunner {
    pub fn new(cmd: String, file: PathBuf) -> Self {
        Self {
            cmd,
            file,
        }
    }

    pub async fn execute(mut self) {

        let cmd = self.cmd.replace("{{file}}", &self.file.as_path().as_os_str().to_string_lossy());

        let mut command = Command::new("bash");
        let command = command.arg("-c").arg(&cmd);

        command.stdin(::std::process::Stdio::piped());
        command.stdout(::std::process::Stdio::piped());
        command.stderr(::std::process::Stdio::piped());

        let mut res = command.spawn().expect("could not spawn command");

        let stdout = res.stdout.take().expect("could not get stdout");
        let h1 = ::tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);

            let mut buf = [0; 4096];
            loop {
                let data = match reader.read(&mut buf).await {
                    Err(_) => break,
                    Ok(0) => break,
                    Ok(size) => &buf[0..size],
                };

                println!("{}", String::from_utf8_lossy(data));
            }
        });

        let stderr = res.stderr.take().expect("could not get stdout");
        let h2 = ::tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buf = [0; 4096];
            loop {
                let data = match reader.read(&mut buf).await {
                    Err(_) => break,
                    Ok(0) => break,
                    Ok(size) => &buf[0..size],
                };

                println!("{}", String::from_utf8_lossy(data));
            }
        });


        let exit_status = res.wait().await.expect("process error ...");
        h1.await.expect("stdout...");
        h2.await.expect("stderr...");

        // println!("process finished with {}", &exit_status);
    }
}