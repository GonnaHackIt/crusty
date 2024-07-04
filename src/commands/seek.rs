use super::*;
use std::time::Duration;

#[poise::command(prefix_command, guild_only, aliases("forward"))]
pub async fn seek(ctx: Context<'_>, time: String) -> Result<(), Error> {
    let Some(seek_time) = text_to_duration(time) else {
        ctx.say("Invalid time format").await?;
        return Ok(());
    };

    let songbird = get_songbird(ctx.serenity_context())
        .await
        .expect("Songbird not registered");

    let Some(call) = songbird.get(ctx.guild_id().unwrap()) else {
        ctx.say("Not in a channel").await?;

        return Ok(());
    };

    let Some(current_track) = call.lock().await.queue().current() else {
        ctx.say("Nothing playing").await?;

        return Ok(());
    };

    let info = current_track.get_info().await?;
    let current_position = info.position;

    let result = current_track.seek_async(current_position + seek_time).await;

    match result {
        Ok(time) => {
            ctx.say(format!("Skipped to {}s", time.as_secs())).await?;
        }
        Err(err) => {
            ctx.say("Skipped beyond the end of song").await?;

            //println!("Error during seeking: {err:?}");
        }
    }

    Ok(())
}

/// Converts time in human format like 50 (50 seconds) or 9:30 (9 minutes and 30 seconds) to Duration.
/// Returns None on invalid format.
fn text_to_duration(time: impl AsRef<str>) -> Option<Duration> {
    let mut parts = time.as_ref().split(":");
    // 4th part is for forcing the right format (2:2:2:2 will currently return None)
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        // If there's no colon we interpret the one string as seconds
        (Some(secs), None, None, None) => secs
            .parse::<u64>()
            .ok()
            .map(|secs| Duration::from_secs(secs)),
        // If there is a colon we interpret two strings as minutes and seconds
        (Some(mins), Some(secs), None, None) => {
            let mins = mins.parse::<u64>().ok()?;
            let secs = secs.parse::<u64>().ok()?;
            if secs > 59 || mins > 59 {
                return None;
            }
            Some(Duration::from_secs(mins * 60 + secs))
        }
        (Some(hours), Some(mins), Some(secs), None) => {
            let hours = hours.parse::<u64>().ok()?;
            let mins = mins.parse::<u64>().ok()?;
            let secs = secs.parse::<u64>().ok()?;
            if secs > 59 || mins > 59 || hours > 23 {
                return None;
            }
            Some(Duration::from_secs(hours * 3600 + mins * 60 + secs))
        }
        _ => None,
    }
}
