use super::*;

// creating separate function for reusing
pub async fn join_channel(ctx: Context<'_>) -> Result<(), Error> {
    let songbird = get_songbird(ctx.serenity_context())
        .await
        .expect("Songbird not registered");

    // try to get channel that user is connected to
    let channel_id = {
        let guild = ctx.guild().expect("guild only command");

        guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice| voice.channel_id)
    };

    let Some(channel_id) = channel_id else {
        ctx.say("You must be in a voice channel").await?;

        return Ok(());
    };

    let call = songbird.join(ctx.guild_id().unwrap(), channel_id).await?;

    call.lock().await.deafen(true).await?;

    Ok(())
}

#[poise::command(prefix_command, guild_only, aliases("revive"))]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    join_channel(ctx).await?;

    Ok(())
}
