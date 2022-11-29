mod config;
mod crawler;
mod helpers;
mod storage;
mod types;
mod ui;

use eyre::Result;
use tokio::{
    sync::mpsc::{channel, Receiver},
    task::JoinHandle,
};
use tracing::info;

use config::Config;
use storage::Storage;

use std::path::Path;

use crate::types::Message;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    // let config = config::Config::open().ok();

    let storage_path = config::Config::archive_path();
    let storage = Storage::open(&storage_path);

    ui::run_ui(storage.ok());

    // println!("Storage: {}", storage_path.display());
    // let storage = Storage::open(&storage_path);
    // let cmd = match &storage {
    //     Ok(existing) => clap::Command::new(name)
    //         .bin_name(name)
    //         .after_help(format!(
    //             "Found an existing storage at {} for {}",
    //             existing.root_folder.display(),
    //             existing.data().profile.screen_name
    //         ))
    //         .subcommand_required(true)
    //         .subcommand(clap::command!("sync"))
    //         .subcommand(clap::command!("inspect"))
    //         .subcommand(clap::command!("ui")),
    //     Err(_) => clap::Command::new(name)
    //         .bin_name(name)
    //         .after_help(format!(
    //             "Found no existing storage at {}",
    //             Config::archive_path().display()
    //         ))
    //         .subcommand_required(false)
    //         .subcommand(clap::command!("crawl"))
    //         .subcommand(clap::command!("ui")),
    // };

    // let matches = cmd.get_matches();
    // match (matches.subcommand(), storage) {
    //     (Some(("crawl", _)), _) => action_crawl(&config, &storage_path).await?,
    //     (Some(("inspect", _)), Ok(storage)) => action_inspect(&storage).await?,
    //     (Some(("ui", _)), Ok(storage)) => action_ui(Some(storage)).await?,
    //     (Some(("ui", _)), Err(_)) => action_ui(None).await?,
    //     (Some(("sync", _)), Ok(storage)) => action_sync(&config, storage).await?,
    //     _ => unreachable!("clap should ensure we don't get here"),
    // };

    Ok(())
}

async fn action_crawl(config: &Config, storage_path: &Path) -> Result<()> {
    info!("Crawling");
    let (sender, receiver) = channel(256);

    crawler::crawl_new_storage(config.clone(), storage_path, sender).await?;
    let storage = log_task(receiver).await??;
    storage.save();
    action_inspect(&storage).await?;
    Ok(())
}

async fn action_sync(config: &Config, storage: Storage) -> Result<()> {
    let mut config = config.clone();
    config.is_sync = true;
    info!("Crawling");
    let (sender, receiver) = channel(256);
    crawler::crawl_into_storage(config.clone(), storage, sender).await?;
    let storage = log_task(receiver).await??;
    storage.save()?;
    action_inspect(&storage).await?;
    Ok(())
}

fn log_task(mut receiver: Receiver<Message>) -> JoinHandle<Result<Storage>> {
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                Message::Initial => {
                    info!("Starting");
                }
                Message::Finished(m) => {
                    return Ok(m);
                }
                Message::Loading(n) => {
                    info!("Loading {n:?}");
                }
                Message::Error(error) => {
                    return Err(error);
                }
            }
        }
        Err(eyre::eyre!("Invalid Loop Break"))
    })
}

async fn action_inspect(storage: &Storage) -> Result<()> {
    println!("tweets: {}", storage.data().tweets.len());
    println!("mentions: {}", storage.data().mentions.len());
    println!("responses: {}", storage.data().responses.len());
    println!("profiles: {}", storage.data().profiles.len());
    println!("followers: {}", storage.data().followers.len());
    println!("follows: {}", storage.data().follows.len());
    println!("lists: {}", storage.data().lists.len());
    for list in storage.data().lists.iter() {
        println!(" {} members: {}", list.name, list.members.len());
    }
    println!("media: {}", storage.data().media.len());
    Ok(())
}

async fn action_ui(_storage: Option<Storage>) -> Result<()> {
    // action_inspect(&storage).await?;
    // FIXME:
    // This will re-open the storage
    // std::mem::drop(storage);
    ui::run_ui(None);
    Ok(())
}

pub fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter::EnvFilter, fmt};

    let env_filter = EnvFilter::new("hyper=info,twittalypse=debug");

    let collector = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(env_filter);

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}
