use std::path::PathBuf;
use async_channel::{Receiver, Sender};
use walkdir::WalkDir;
use structopt::StructOpt;
use tokio::task::JoinHandle;
use crate::command_runner::CommandRunner;

mod command_runner;

fn get_files(path : &str) -> Vec<PathBuf> {

    let mut buf = vec![];

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();

        if !entry.file_type().is_file() {
            continue;
        }

        buf.push(entry.into_path());
    }

    // dbg!(&buf);

    buf
}

async fn vec_to_queue(vec: Vec<PathBuf>) -> (Sender<PathBuf>, Receiver<PathBuf>) {
    let channel = ::async_channel::unbounded::<PathBuf>();

    for item in vec.into_iter() {
        channel.0.send(item).await.expect("could not put in channel...");
    }

    channel
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long = "files", default_value = ".")]
    pub files: String,
    #[structopt(long = "threads", default_value = "3")]
    pub threads: u32,
    pub command : String,
}

#[tokio::main]
async fn main() {
    let opt : Opt = Opt::from_args();

    let (_, file_provider) = vec_to_queue(get_files(&opt.files)).await;

    let mut workers = vec![];
    for i in 1..=opt.threads {
        let worker_file_provider = file_provider.clone();
        let worker_opt = opt.clone();
        workers.push(::tokio::spawn(async move {
            do_command(worker_opt, worker_file_provider).await;
        }));
    }

    for worker in workers.into_iter() {
        worker.await.expect("worker failed")
    }
}


async fn do_command(worker_opt: Opt, worker_file_provider : Receiver<PathBuf>) {

    let mut i : u32 = 0;

    loop {
        let item = match worker_file_provider.recv().await {
            Ok(i) => i,
            Err(e) => {
                break;
            }
        };

        i = i + 1;

        let command_runner = CommandRunner::new(worker_opt.command.clone(), item);
        command_runner.execute().await;
    }

    // println!("commandrunner finished");
}