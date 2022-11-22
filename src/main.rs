mod crawler;
mod helpers;
mod storage;
mod types;

use crate::types::Message;
use tracing::info;

// mod ui;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use storage::Storage;

use eyre::{bail, Result};
use tokio::{sync::mpsc, try_join};

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    let config = helpers::Config::load().await?;

    let Ok(storage_path) = PathBuf::from_str(&format!("archive_{}", config.user_id)) else { bail!("Invalid Path") };

    info!("Found User {}", config.screen_name);

    let storage = match Storage::open(&storage_path) {
        Ok(existing) => existing,
        Err(e) => {
            info!("Crawling: Could not open storage: {e:?}.");
            let storage = crawl_into_storage(config.clone(), &storage_path).await?;
            println!("Saved data to {}", storage_path.display());
            storage
        }
    };

    println!("tweets: {}", storage.data().tweets.len());
    println!("mentions: {}", storage.data().mentions.len());
    println!("responses: {}", storage.data().responses.len());
    println!("profiles: {}", storage.data().profiles.len());
    println!("followers: {}", storage.data().followers.len());
    println!("follows: {}", storage.data().follows.len());
    println!("lists: {}", storage.data().lists.len());
    println!("media: {}", storage.data().media.len());

    Ok(())
}

async fn crawl_into_storage(config: helpers::Config, storage_path: &Path) -> Result<Storage> {
    let Ok(user_container) = egg_mode::user::lookup([config.user_id], &config.token).await else { bail!("Could not find user") };
    let Some(user) = user_container.response.first() else { bail!("Empty User Response") };
    let storage = Storage::new(user.clone(), storage_path)?;

    // crawl
    let (sender, mut receiver) = mpsc::channel(32);
    let output_task = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
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
        Err(eyre::eyre!("Invalid Loop Brak"))
    });

    let crawl_task = tokio::spawn(async move {
        match crawler::fetch(&config, storage, sender.clone()).await {
            Ok(_) => (),
            Err(e) => {
                if let Err(e) = sender.send(Message::Error(e)).await {
                    println!("Could not close channel for error  {e:?}");
                }
            }
        }
    });

    let (storage, _) = try_join!(output_task, crawl_task)?;
    let storage = storage?;

    storage.save()?;
    Ok(storage)
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
