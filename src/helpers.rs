use crate::crawler::DownloadInstruction;
use egg_mode::tweet::Tweet;

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
