mod crawler;
mod helpers;
mod storage;
mod types;
// mod ui;

use std::{path::PathBuf, str::FromStr};
use storage::Storage;

use eyre::{bail, Result};
use tokio::{sync::mpsc, try_join};

fn main() {
    setup_tracing();
    helpers::apiv2_helper::login().unwrap();
}

#[tokio::main]
async fn xmain() -> Result<()> {
    setup_tracing();
    info!("Parse Config");
    let config = helpers::Config::load().await;

    let Ok(storage_path) = PathBuf::from_str("test") else { bail!("Invalid Path") };

    // For now, we always recreate the storage
    info!("Found User {}", config.screen_name);
    let Ok(user_container) = egg_mode::user::lookup([config.user_id], &config.token).await else { bail!("Could not find user") };
    let Some(user) = user_container.response.first() else { bail!("Empty User Response") };
    let storage = Storage::new(user.clone(), &storage_path);

    // crawl
    let (sender, mut receiver) = mpsc::channel(32);
    let output_task = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                Message::Finished(m) => {
                    println!("tweets: {}", m.data().tweets.len());
                    println!("mentions: {}", m.data().mentions.len());
                    println!("responses: {}", m.data().responses.len());
                    println!("profiles: {}", m.data().profiles.len());
                    println!("followers: {}", m.data().followers.len());
                    println!("follows: {}", m.data().follows.len());
                    println!("lists: {}", m.data().lists.len());
                    println!("media: {}", m.data().media.len());
                }
                Message::Loading(n) => {
                    info!("Loading {n:?}");
                }
            }
        }
    });

    let crawl_task = tokio::spawn(async move {
        match crawler::fetch(&config, storage, sender).await {
            Ok(_) => (),
            Err(e) => {
                println!("Error during crawling: {e:?}");
                panic!();
            }
        }
    });

    try_join!(output_task, crawl_task)?;

    // let storage = if storage_path.exists() {
    //     Storage::open(&storage_path)?
    // } else {
    // };

    Ok(())
}

use tracing::{info, trace};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
};

use crate::types::Message;

pub fn setup_tracing() {
    let env_filter = EnvFilter::new("hyper=info,twittalypse=debug");

    let collector = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(env_filter);

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}
