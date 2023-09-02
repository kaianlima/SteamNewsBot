use std::sync::Arc;

#[derive(Clone)]
pub struct Data(pub Arc<DataInner>);

pub struct DataInner {
    pub discord_guild_id: String,
    pub ds_token: String,
    pub reqwest: reqwest::Client,
    pub steam_token: String,
}

pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, CommandError>;
pub type CommandError = Error;
pub type CommandResult<E=Error> = Result<(), E>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;