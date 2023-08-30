use reqwest::Client;
use serde::Deserialize;
use std::{fmt::Display, collections::HashMap};
use tracing::info;

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
    date: i32,
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
    game: &str,
	api_key: &str,
    client: &Client,
) -> Result<SteamApp, Box<dyn std::error::Error>> {
	// Endpoints we will use
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamApps";
    const METHOD: &str = "GetAppList";
    const VERSION: &str = "v0002";

    let url = format!("{}{}/{}/{}", API_URL, INTERFACE, METHOD, VERSION);

    let response: MainAppList = client.get(url)
        .send()
        .await?
        .json()
        .await?;

    let steamapp = response.applist.apps.into_iter().find(|x| x.name == game).ok_or_else(|| CouldNotFindApp {
        game: game.to_owned(),
    })?;

    println!("{:#?}", steamapp);
    info!("{:#?}", steamapp);

    Ok(steamapp)
}

pub async fn get_news(
    game: &str,
	api_key: &str,
    client: &Client,
) -> Result<NewsItems, Box<dyn std::error::Error>> {
	// Endpoints we will use
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamNews";
    const METHOD: &str = "GetNewsForApp";
    const VERSION: &str = "v0002";
    const COUNT: &str = "999";
    const MAXLENGTH: &str = "300";

    let steamapp = get_app(game, api_key, client).await?;

    let url = format!("{}{}/{}/{}/?appid={}&count={}&maxlength={}", API_URL, INTERFACE, METHOD, VERSION, steamapp.appid, COUNT, MAXLENGTH);

	let response: MainAppNews = client.get(url)
        .send()
        .await?
        .json()
        .await?;

    let appnews = response.appnews.newsitems.into_iter().filter(|x| x.feedname == "steam_community_announcements").next().ok_or_else(|| CouldNotFindNews{})?;

    println!("{:#?}", appnews);
    info!("{:#?}", appnews);

    Ok(appnews)
}