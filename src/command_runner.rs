use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
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
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            for line in lines.next_line().await {
                let line = match line {
                    None => break,
                    Some(s) => s,
                };

                println!("{:?}", line);
            }
        });

        let stderr = res.stderr.take().expect("could not get stdout");
        let h2 = ::tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            for line in lines.next_line().await {
                let line = match line {
                    None => break,
                    Some(s) => s,
                };

                println!("{:?}", line);
            }
        });


        let exit_status = res.wait().await.expect("process error ...");
        h1.await.expect("stdout...");
        h2.await.expect("stderr...");

        // println!("process finished with {}", &exit_status);
    }
}