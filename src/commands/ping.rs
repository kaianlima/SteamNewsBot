use crate::Context;
use crate::structs::{Command, CommandResult};

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    ctx.say("I'm alive!").await?;
    Ok(())
}

pub fn commands() -> [Command; 1] {
    [ping()]
}