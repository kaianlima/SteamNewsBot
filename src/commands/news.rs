use chrono::{DateTime, Utc, serde::ts_seconds};
use reqwest::Client;
use serde::Deserialize;
use std::fmt::Display;
use tracing::info;

use crate::Context;
use crate::structs::{Command, CommandResult, Error};

#[derive(Deserialize, Debug)]
pub struct MainAppList {
    pub applist: AppList,
}

#[derive(Deserialize, Debug)]
pub struct AppList {
    pub apps: Vec<SteamApp>,
}

#[derive(Deserialize, Debug)]
pub struct SteamApp {
    appid: i32,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct MainAppNews {
    pub appnews: AppNews,
}

#[derive(Deserialize, Debug)]
pub struct AppNews {
    appid: i32,
    pub newsitems: Vec<NewsItems>,
    count: i32,
}

#[derive(Deserialize, Debug)]
pub struct NewsItems {
    gid: String,
    pub title: String,
    url: String,
    is_external_url: bool,
    author: String,
    contents: String,
    feedlabel: String,
    #[serde(with = "ts_seconds")]
    date: DateTime<Utc>,
    feedname: String,
    feed_type: i32,
    appid: i32
}

#[derive(Deserialize, Debug)]
pub struct CouldNotFindApp {
    game: String,
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

pub async fn get_app(
    client: Client,
    game: String,
) -> Result<SteamApp, Error> {
	// API endpoint build
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamApps";
    const METHOD: &str = "GetAppList";
    const VERSION: &str = "v0002";

    let url = format!("{}{}/{}/{}", API_URL, INTERFACE, METHOD, VERSION);

    info!("API call: {:#?}", url);

    let response: MainAppList = client.get(url)
        .send()
        .await?
        .json()
        .await?;

    let steamapp = response
        .applist
        .apps
        .into_iter()
        .find(|x| x.name.to_lowercase().contains(&game.to_lowercase()))
        .ok_or(CouldNotFindApp {
        game: game.to_owned(),
    })?;

    info!("{:#?}", steamapp);

    Ok(steamapp)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn news(
    ctx: Context<'_>, 
    #[description = "Game to lookup news"] game: String, 
    #[description = "News quantity"] quantity: Option<String>
) -> CommandResult {
	// API endpoint build
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamNews";
    const METHOD: &str = "GetNewsForApp";
    const VERSION: &str = "v0002";
    const COUNT: &str = "999";
    const MAXLENGTH: &str = "300";

    let client = ctx.data().0.reqwest.clone();

    let steamapp = get_app(client.clone(), game).await?;

    let count: &str = &quantity.unwrap_or(COUNT.to_string()); 

    let url = format!("{}{}/{}/{}/?appid={}&count={}&maxlength={}", API_URL, INTERFACE, METHOD, VERSION, steamapp.appid, count, MAXLENGTH);

    info!("API call: {:#?}", url);

	let response: MainAppNews = client.get(url)
        .send()
        .await?
        .json()
        .await?;

    let appnews = response
        .appnews
        .newsitems
        .into_iter()
        .filter(|x| x.feedname.to_lowercase() == "steam_community_announcements")
        .next()
        .ok_or(CouldNotFindNews{})?;

    info!("{:#?}", appnews);

    ctx.say(format!("APPNEWS: {:#?}",appnews)).await?;

    Ok(())
}

pub fn commands() -> [Command; 1] {
    [news()]
}