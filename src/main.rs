use std::path::PathBuf;
use std::process::exit;
use async_channel::{Receiver, Sender};
use walkdir::WalkDir;
use tokio::task::JoinHandle;
use crate::command_runner::CommandRunner;
use clap::Parser;

mod command_runner;

fn get_files(path : &str) -> Vec<PathBuf> {

    let mut buf = vec![];

    for entry in WalkDir::new(path).follow_links(false) {
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

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Opt {
    #[arg(long = "files", default_value = ".")]
    pub files: String,
    #[arg(long = "threads", default_value = "3")]
    pub threads: u32,
    pub command : String,
    #[arg(short, long)]
    execute: bool,
}

#[tokio::main]
async fn main() {
    let opt : Opt = Opt::parse();

    let (_, file_provider) = vec_to_queue(get_files(&opt.files)).await;

    let mut workers = vec![];
    for i in 1..=opt.threads {
        let worker_file_provider = file_provider.clone();
        let worker_opt = opt.clone();
        workers.push(::tokio::spawn(async move {
            do_command(worker_opt, worker_file_provider).await
        }));
    }

    let mut exit_codes = vec![];
    for worker in workers.into_iter() {
        exit_codes.extend(worker.await.expect("worker failed"));
    }

    let success_codes = exit_codes.iter().filter(|code| **code == 0).collect::<Vec<_>>();
    let error_codes = exit_codes.iter().filter(|code| **code != 0).collect::<Vec<_>>();

    println!("{} successfull", success_codes.len());
    println!("{} errors", error_codes.len());
}




async fn do_command(worker_opt: Opt, worker_file_provider : Receiver<PathBuf>) -> Vec<u32> {

    let mut i : u32 = 0;
    let mut exit_codes = vec![];

    loop {
        let item = match worker_file_provider.recv().await {
            Ok(i) => i,
            Err(e) => {
                break;
            }
        };

        i = i + 1;

        let command_runner = CommandRunner::new(worker_opt.clone(), item);
        exit_codes.push(command_runner.execute().await);
    }

    println!("commandrunner finished");

    exit_codes
}