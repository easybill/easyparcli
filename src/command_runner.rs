use std::path::PathBuf;
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use crate::Opt;

pub struct CommandRunner {
    file: PathBuf,
    cmd: String,
    execute: bool,
}

impl CommandRunner {
    pub fn new(opt: Opt, file: PathBuf) -> Self {
        Self {
            cmd: opt.command,
            file,
            execute: opt.execute,
        }
    }

    pub async fn execute(mut self) -> u32 {
        let cmd = self.cmd
            .replace("{{file}}", &{
                let mut file = self.file.as_path().as_os_str().to_string_lossy().into_owned();

                if file.starts_with("./") {
                    file = file.replacen("./", "", 1);
                }

                file
            })
            .replace("{{directory}}", &{
                match self.file.parent() {
                    None => "".to_string(),
                    Some(parent) => {
                        let mut file = parent.as_os_str().to_string_lossy().into_owned();

                        if file.starts_with("./") {
                            file = file.replacen("./", "", 1);
                        }

                        if &file == "." {
                            file = "".to_string();
                        }

                        file
                    },
                }
            });

        if !self.execute {
            println!("execute: {}", &cmd);
            return 0;
        }


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

        println!("process finished with {}, command: {}", &exit_status, &cmd);
        exit_status.code().expect("no exit code") as u32
    }
}