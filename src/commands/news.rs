use chrono::{DateTime, Utc, serde::ts_seconds};
use reqwest::Client;
use serde::Deserialize;
use serenity::model::prelude::application_command::CommandDataOption;
use std::fmt::Display;
use tracing::{info, instrument};

use crate::Bot;

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
    client: &Client,
    game: &str,
) -> Result<SteamApp, Box<dyn std::error::Error>> {
    info!("Starting get_app function");

	// Endpoints we will use
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
    info!("Ending get_app function");

    Ok(steamapp)
}

pub async fn get_news(
    client: &Client,
    game: &str,
    quantity: &str,
) -> Result<NewsItems, Box<dyn std::error::Error>> {
    info!("Starting get_news function");
	// Endpoints we will use
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamNews";
    const METHOD: &str = "GetNewsForApp";
    const VERSION: &str = "v0002";
    const COUNT: &str = "999";
    const MAXLENGTH: &str = "300";

    let steamapp = get_app(client, game).await?;

    let count: &str = if !quantity.is_empty() { quantity } else { COUNT }; 

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
    info!("Ending get_news function");

    Ok(appnews)
}

pub async fn run(_options: &[CommandDataOption], client_req: &Client) -> String {
    let value1 = _options
        .iter()
        .find(|opt| opt.name == "game")
        .cloned()
        .unwrap()
        .value
        .unwrap();

        let value2 = _options
            .iter()
            .find(|opt| opt.name == "quantity")
            .cloned()
            .unwrap()
            .value
            .unwrap();
    
        let game = value1.as_str().unwrap();
        let quantity = value2.as_str().unwrap();
        let result = get_news(&client_req, game, quantity).await;
    
        match result {
            Ok(appnews) => format!(
                "Title: {:#?}",
                appnews
            ),
            Err(err) => {
                format!("Err: {}", err)
            }
        }
}