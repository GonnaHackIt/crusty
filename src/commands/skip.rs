use super::*;

#[poise::command(prefix_command, guild_only, aliases("fs", "s"))]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let songbird = ctx.data().songbird.clone();

    let Some(call) = songbird.get(ctx.guild_id().unwrap()) else {
        ctx.say("Not in a channel").await?;

        return Ok(());
    };

    let _ = call.lock().await.queue().skip();

    Ok(())
}
