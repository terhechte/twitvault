use crate::{config::Config, crawler::DownloadInstruction};
use egg_mode::tweet::Tweet;
use tracing::warn;

pub fn media_in_tweet(tweet: &Tweet) -> Option<Vec<DownloadInstruction>> {
    let Some(entities) = &tweet.extended_entities else { return None };

    let mut output = Vec::new();

    for media in &entities.media {
        match &media.video_info {
            Some(n) => {
                let mut selected_variant = n.variants.first();
                for variant in &n.variants {
                    match (
                        variant.content_type.subtype(),
                        &selected_variant.map(|e| e.bitrate),
                    ) {
                        (mime::MP4, Some(bitrate)) if bitrate < &variant.bitrate => {
                            selected_variant = Some(variant)
                        }
                        _ => (),
                    }
                }
                let Some(variant) = selected_variant else { continue };
                output.push(DownloadInstruction::Movie(
                    variant.content_type.clone(),
                    variant.url.clone(),
                ))
            }
            None => output.push(DownloadInstruction::Image(media.media_url_https.clone())),
        }
    }

    Some(output)
}

pub async fn delete_tweet(tweet_id: u64, config: &Config) -> Result<bool, String> {
    egg_mode::tweet::delete(tweet_id, &config.token)
        .await
        .map(|_| true)
        .map_err(|e| {
            warn!("Could not delete tweet: {e:?}");
            format!("{e:?}")
        })
}
