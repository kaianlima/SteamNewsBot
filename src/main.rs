mod commands;

use anyhow::anyhow;
use serenity::model::prelude::command::{CommandOptionType, Command};
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::{async_trait, model::prelude::GuildId};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{debug, error, info};

pub struct Bot {
    steam_api_key: String,
    client_req: reqwest::Client,
	discord_guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    // `interaction_create` runs when the user interacts with the bot
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // check if the interaction is a command
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!("Received command interaction: {:#?}", command);

            let response_content =
                match command.data.name.as_str() {
                    "ping" => commands::ping::run(&command.data.options),
                    "news" => commands::news::run(&command.data.options, &self.client_req).await,
                    command => unreachable!("Unknown command: {}", command),
                };

            if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(response_content))
                })
                .await
            {
                info!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
       
        // Register guild commands
        let commands = GuildId::set_application_commands(&self.discord_guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| { command.name("ping").description("A ping command") })
                .create_application_command(|command| {
                    command
                        .name("news")
                        .description("Display the news")
                        .create_option(|option| {
                            option
                                .name("game")
                                .description("Game to lookup news")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("quantity")
                                .description("News quantity")
                                .kind(CommandOptionType::String)
                                .required(false)
                        })
                })
        }).await.unwrap();

        /* Register global commands
        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::global_command::register(command)
        }).await.unwrap();
        */
        
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let steam_api_key = if let Some(steam_api_key) = secret_store.get("STEAM_API_KEY") {
        steam_api_key
    } else {
        return Err(anyhow!("'STEAM_API_KEY' was not found").into());
    };

    let discord_guild_id = if let Some(discord_guild_id) = secret_store.get("DISCORD_GUILD_ID") {
        discord_guild_id
    } else {
        return Err(anyhow!("'DISCORD_GUILD_ID' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES;

    let client = Client::builder(&token, intents)
        .event_handler(Bot {
            steam_api_key,
            client_req: reqwest::Client::new(),
			discord_guild_id: GuildId(discord_guild_id.parse().unwrap())
        })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
