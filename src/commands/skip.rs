use super::*;

#[poise::command(prefix_command, guild_only, aliases("fs", "s"))]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let songbird = get_songbird(ctx.serenity_context())
        .await
        .expect("Songbird not registered");

    let Some(call) = songbird.get(ctx.guild_id().unwrap()) else {
        ctx.say("Not in a channel").await?;

        return Ok(());
    };

    let _ = call.lock().await.queue().skip();

    Ok(())
}
