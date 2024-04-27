use super::*;

#[poise::command(prefix_command, guild_only, aliases("forward"))]
pub async fn seek(ctx: Context<'_>, secs: u64) -> Result<(), Error> {
    let seek_time = std::time::Duration::from_secs(secs);

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
            ctx.say(format!("Skipped to {}", time.as_secs())).await?;
        }
        Err(err) => {
            ctx.say("Skipped beyond the end of song").await?;

            //println!("Error during seeking: {err:?}");
        }
    }

    Ok(())
}
