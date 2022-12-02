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
use tracing::{info, warn};

use config::Config;
use storage::Storage;

use std::path::Path;

use crate::types::Message;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    let name = "TwitVault";
    let storage_path = config::Config::archive_path();
    println!("Try opening Storage: {}", storage_path.display());

    let config = config::Config::open().ok();

    let storage = Storage::open(&storage_path);
    let cmd = match &storage {
        Ok(existing) => clap::Command::new(name)
            .bin_name(name)
            .after_help(format!(
                "Found an existing storage at {} for {}",
                existing.root_folder.display(),
                existing.data().profile.screen_name
            ))
            .subcommand_required(false)
            .subcommand(clap::command!("sync"))
            .subcommand(clap::command!("inspect")),
        Err(_) => clap::Command::new(name)
            .bin_name(name)
            .after_help(format!(
                "Found no existing storage at {}",
                Config::archive_path().display()
            ))
            .subcommand_required(false)
            .subcommand(clap::command!("crawl")),
    };

    let matches = cmd.get_matches();
    match (matches.subcommand(), storage, config) {
        // Try to crawl with a pre-defined config
        (Some(("crawl", _)), Err(_), Some(config)) => action_crawl(&config, &storage_path).await?,
        // If there's no config, perform the login dance in the terminal, then crawl
        (Some(("crawl", _)), Err(_), None) => {
            let config = Config::load().await.expect("Could not create config");
            action_crawl(&config, &storage_path).await?
        }
        // For an existing storage, inspect it
        (Some(("inspect", _)), Ok(storage), _) => action_inspect(&storage).await?,
        // For an existing storage, sync it
        (Some(("sync", _)), Ok(storage), Some(config)) => action_sync(&config, storage).await?,
        // In all other cases, show the UI
        (_, optional_storage, optional_config) => {
            action_ui(optional_storage.ok(), optional_config).await?
        }
    };

    Ok(())
}

async fn action_crawl(config: &Config, storage_path: &Path) -> Result<()> {
    info!("Crawling");
    let (sender, receiver) = channel(256);

    crawler::crawl_new_storage(config.clone(), storage_path, sender).await?;
    let storage = log_task(receiver).await??;
    if let Err(e) = storage.save() {
        warn!("Could not save storage {e:?}");
    }
    action_inspect(&storage).await?;
    Ok(())
}

async fn action_sync(config: &Config, storage: Storage) -> Result<()> {
    info!("Syncing");
    let mut config = config.clone();
    config.is_sync = true;
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

async fn action_ui(storage: Option<Storage>, config: Option<Config>) -> Result<()> {
    ui::run_ui(storage, config);
    Ok(())
}

pub fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter::EnvFilter, fmt};

    let env_filter = EnvFilter::new("hyper=info,twitvault=debug");

    let collector = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(env_filter);

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}
