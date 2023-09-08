use reqwest::Client;
use std::time::Duration;
use tracing::info;

use crate::Context;
use crate::funcs::{search_in, order_apps};
use crate::structs::{Command, CommandResult, CouldNotFindApp, CouldNotFindNews, Error, MainAppList, MainAppNews, SteamApp};

pub async fn get_app(
    client: Client,
    game: String,
) -> Result<Vec<SteamApp>, Error> {
	// API endpoint var
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
        .filter(|x| x.name.to_lowercase().contains(&game.to_lowercase()))
        .collect::<Vec<SteamApp>>();

    info!("Found {:#?} apps", steamapp.len());

    Ok(steamapp)
}

#[poise::command(prefix_command, slash_command, reuse_response, track_edits)]
pub async fn news(
    ctx: Context<'_>, 
    game: String, 
    quantity: Option<String>
) -> CommandResult {
    info!("Commands parameters: {{Game: {:#?} - Quantity: {:#?}}}", &game, &quantity.clone().unwrap_or("Not provided, set to default".to_string()));

    let client = ctx.data().0.reqwest.clone();
    let steamapp: &mut Vec<SteamApp> = &mut get_app(client.clone(), game.clone()).await?;
    let (steamapps, app_searched) = search_in(steamapp, game.clone());

    //steamapp.clear();

    let appid = if app_searched.is_some() {
        app_searched.unwrap().appid.to_string()
    } else {
        order_apps(steamapps);

        info!("Ordered Apps: {:#?}", steamapps);
        
        let mut app_menu_options: Vec<poise::serenity_prelude::CreateSelectMenuOption> = Vec::new();
        let mut apps_iterator = steamapps.iter().peekable();
        let mut count_iter = 0;
        while let Some(app) = apps_iterator.next() {
            app_menu_options.push(poise::serenity_prelude::CreateSelectMenuOption::new(app.name.to_string(), app.appid.to_string()));
            count_iter += 1;
            if apps_iterator.peek().is_none() || count_iter == MAX_SELECT_OPTION {
                break;
            }
        }

        let reply = ctx.send(|builder| {
            builder.content("Select game")
            .ephemeral(true)
            .components(|components| {
                components.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("game_select");
                        menu.placeholder("No game selected");
                        menu.options(|f| {
                            f.set_options(app_menu_options)
                        })
                    })

                })
            })
        }).await?;

        let interaction =
            match reply.message()
            .await?
            .await_component_interaction(ctx)
            .timeout(Duration::from_secs(60 * 3))
            .await {
                Some(x) => {
                    reply.edit(ctx, |b| {
                        b.content(format!("Game {:?} selected", x.data.values[0]))
                    })
                    .await
                    .unwrap();
                    x
                },
                None => {
                    reply.delete(ctx).await?;
                    return Ok(());
                },
            };

        //info!("Data: {:#?}", &interaction);

        interaction.data.values[0].clone()
    };

    // API endpoint var
    const API_URL: &str = "http://api.steampowered.com/";
    const INTERFACE: &str = "ISteamNews";
    const METHOD: &str = "GetNewsForApp";
    const VERSION: &str = "v0002";
    const COUNT: &str = "999";
    const MAX_LENGTH: &str = "300";
    const MAX_SELECT_OPTION: usize = 10;
    let count: &str = &quantity.unwrap_or(COUNT.to_string()); 

    let url = format!("{}{}/{}/{}/?appid={}&count={}&maxlength={}", API_URL, INTERFACE, METHOD, VERSION, appid, count, MAX_LENGTH);

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
        .find(|x| x.feedname.to_lowercase() == "steam_community_announcements")
        .ok_or(CouldNotFindNews{})?;

    info!("App News: {:#?}", appnews);

    // interaction.edit(&ctx, |response| {
    //     response.content(format!("APPNEWS: {:#?}", appnews))
    // })
    // .await?;

    ctx.send(|builder| {
        builder
        .content(format!("APPNEWS: {:#?}", appnews))
        .reply(false)
    })
    .await?;

    //reply.delete(ctx).await.unwrap();

    // let mut msg = interaction.message.clone();
    // msg.edit(ctx, |m| m.content(format!("APPNEWS: {:#?}",appnews))).await?;

    // interaction.create_interaction_response(ctx, |ir| {
    //     ir.kind(InteractionResponseType::DeferredUpdateMessage)
    // })
    // .await?;

    // interaction.create_interaction_response(ctx_discord, |response| {
    //     response.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|data| {
    //         data.content(format!("APPNEWS: {:#?}",appnews)).components(|c|{c})
    //     })
    // })
    // .await
    // .unwrap();

    //reply.message().await.unwrap().delete(ctx).await.unwrap();

    Ok(())
}

pub fn commands() -> [Command; 1] {
    [news()]
}