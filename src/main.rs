mod commands;
mod structs;

use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{GatewayIntents, GuildId};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::{sync::{Arc, Mutex}, time::Duration};
use tracing::{error, info};
use structs::{Context, Data, DataInner, Error};

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let ds_token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let steam_token = if let Some(steam_token) = secret_store.get("STEAM_API_KEY") {
        steam_token
    } else {
        return Err(anyhow!("'STEAM_API_KEY' was not found").into());
    };

    let discord_guild_id = if let Some(discord_guild_id) = secret_store.get("DISCORD_GUILD_ID") {
        discord_guild_id
    } else {
        return Err(anyhow!("'DISCORD_GUILD_ID' was not found").into());
    };

    let reqwest = reqwest::Client::new();

    let data = Data(Arc::new(DataInner {
        ds_token: ds_token.clone(), steam_token, discord_guild_id, reqwest
    }));

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES;

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!sn ".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(60 * 5))),
            ..Default::default()
        },
        /// The global error handler for all error cases that may occur
        on_error: |error| {
            Box::pin(async move {
                match error {
                    poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
                    poise::FrameworkError::Command { error, ctx } => {
                        error!("Error in command `{}`: {:?}", ctx.command().name, error,);
                    }
                    poise::FrameworkError::ArgumentParse { error, .. } => {
                        if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                            error!("Found a RoleParseError: {:?}", error);
                        } else {
                            error!("Not a RoleParseError :(");
                        }
                    }
                    other => {
                        if let Err(e) = poise::builtins::on_error(other).await {
                            error!("Error while handling error: {}", e)
                        }
                    },
                }
            })
        },
        /// This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        /// Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        ..poise::FrameworkOptions::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .token(ds_token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
