use chrono::{DateTime, Utc, serde::ts_seconds};
use serde::Deserialize;
use std::{fmt::Display, sync::Arc};

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

#[derive(Deserialize, Debug)]
pub struct MainAppList {
    pub applist: AppList,
}

#[derive(Deserialize, Debug)]
pub struct AppList {
    pub apps: Vec<SteamApp>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SteamApp {
    pub appid: i32,
    pub name: String,
    #[serde(skip_deserializing)]
    pub search_score: i32,
}

#[derive(Deserialize, Debug)]
pub struct MainAppNews {
    pub appnews: AppNews,
}

#[derive(Deserialize, Debug)]
pub struct AppNews {
    pub appid: i32,
    pub newsitems: Vec<NewsItems>,
    pub count: i32,
}

#[derive(Deserialize, Debug)]
pub struct NewsItems {
    pub gid: String,
    pub title: String,
    pub url: String,
    pub is_external_url: bool,
    pub author: String,
    pub contents: String,
    pub feedlabel: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
    pub feedname: String,
    pub feed_type: i32,
    pub appid: i32
}

#[derive(Deserialize, Debug)]
pub struct CouldNotFindApp {
    pub game: String,
}

#[derive(Deserialize, Debug)]
pub struct CouldNotFindNews {}

impl Display for CouldNotFindApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find game '{}'", self.game)
    }
}

impl Display for CouldNotFindNews {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find news")
    }
}

impl std::error::Error for CouldNotFindApp {}

impl std::error::Error for CouldNotFindNews {}