mod config;
mod crawler;
mod helpers;
mod importer;
mod search;
mod storage;
mod types;
mod ui;

use clap::{ArgMatches, Command};
use eyre::{bail, Result};
use tokio::{
    sync::mpsc::{channel, Receiver},
    task::JoinHandle,
};
use tracing::{info, warn};

use config::Config;
use storage::Storage;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::types::Message;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    let name = "TwitVault";

    // check if we have a path to a custom storage
    let raw_args: Vec<_> = std::env::args().collect();
    let (config, storage, storage_path) =
        match (raw_args.get(1).map(|e| e.as_str()), raw_args.get(2)) {
            (Some("--custom-archive"), Some(custom)) => {
                let custom_path = PathBuf::from_str(custom)?;
                if !custom_path.exists() {
                    std::fs::create_dir_all(&custom_path)
                        .expect("Expect to be able to create the data directory");
                }
                println!("Try opening Storage: {}", &custom_path.display());
                let config = config::Config::open(Some(custom_path.clone())).ok();
                let storage = Storage::open(custom_path.clone());
                (config, storage, custom_path)
            }
            _ => {
                let storage_path = config::Config::storage_path(None);
                if !storage_path.exists() {
                    std::fs::create_dir_all(&storage_path)
                        .expect("Expect to be able to create the data directory");
                }
                println!("Try opening Storage: {}", storage_path.display());
                let config = config::Config::open(None).ok();
                let storage = Storage::open(&storage_path);
                (config, storage, storage_path)
            }
        };

    let cmd = match &storage {
        Ok(existing) => clap::Command::new(name)
            .bin_name(name)
            .after_help(format!(
                "Found an existing storage at {} for {}",
                existing.root_folder.display(),
                existing.data().profile.screen_name
            ))
            .arg(clap::Arg::new("custom-archive")
            .long("custom-archive")
            .help("Absolute path to a different archive folder tahn the default")
            .required(false))
            .subcommand_required(false)
            .subcommand(clap::command!("sync"))
            .subcommand(
                Command::new("import")
                    .arg(clap::Arg::new("archive-path").required(true).short('c')),
            )
            .subcommand(clap::command!("inspect")),
        Err(_) => clap::Command::new(name)
            .bin_name(name)
            .after_help(format!(
                "Found no existing storage at {}",
                storage_path.display()
            ))
            .subcommand_required(false)
            .subcommand(
                Command::new("crawl")
                    .arg(clap::Arg::new("custom-user")
                    .help("Don't crawl the data of the authenticated user, but instead of the given custom-user which is the Twitter user id such as 6473172")
                    .required(false).short('u')),
            ),
    };

    let matches = cmd.get_matches();
    match (matches.subcommand(), storage, config) {
        // Try to crawl with a pre-defined config
        (Some(("crawl", custom)), Err(_), Some(config)) => {
            action_crawl(&config, &storage_path, custom).await?
        }
        // If there's no config, perform the login dance in the terminal, then crawl
        (Some(("crawl", custom)), Err(_), None) => {
            let config = Config::load(Some(storage_path.clone()))
                .await
                .expect("Could not create config");
            action_crawl(&config, &storage_path, custom).await?
        }
        // Import a Twitter archive
        (Some(("import", archive)), Ok(storage), Some(config)) => {
            action_import(&config, storage, archive).await?
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

async fn action_import(config: &Config, storage: Storage, matches: &ArgMatches) -> Result<()> {
    let Some(path) = matches.get_one::<String>("archive-path") else {
        bail!("Missing parameter --archive-path [...]")
    };
    let storage = importer::import_archive(storage, config, path).await?;
    storage.save()?;
    action_inspect(&storage).await?;
    Ok(())
}

async fn action_crawl(config: &Config, _storage_path: &Path, matches: &ArgMatches) -> Result<()> {
    let user_id = match matches
        .get_one::<String>("custom-user")
        .map(|n| n.parse::<u64>())
    {
        Some(Err(e)) => {
            bail!("The given custom-user could not be parsed: {e:?}")
        }
        Some(Ok(n)) => n,
        None => config.user_id(),
    };
    info!("Crawling");
    let (sender, receiver) = channel(256);

    // In custom-user mode, disable responses and mentions
    let mut config = config.clone();
    if user_id != config.user_id() {
        let mut options = config.crawl_options().clone();
        options.mentions = false;
        options.tweet_responses = false;
        config.set_crawl_options(&options);
    }

    crawler::crawl_new_storage(config, sender, user_id).await?;
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
    crawler::crawl_into_storage(config.user_id(), config.clone(), storage, sender).await?;
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
