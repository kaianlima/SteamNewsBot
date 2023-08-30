use anyhow::anyhow;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::{async_trait, model::prelude::GuildId};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};
mod news;

struct Bot {
    steam_api_key: String,
    client_req: reqwest::Client,
	discord_guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
       
       // add "/hello" command to the bot
       GuildId::set_application_commands(&self.discord_guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| { command.name("hello").description("Say hello") })
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
                })
       }).await.unwrap();
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    // `interaction_create` runs when the user interacts with the bot
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    // check if the interaction is a command
    if let Interaction::ApplicationCommand(command) = interaction {

        let response_content =
            match command.data.name.as_str() {
                "hello" => "hello".to_owned(),
                "news" => {
                    let argument = command
                        .data
                        .options
                        .iter()
                        .find(|opt| opt.name == "game")
                        .cloned();
                
                    let value = argument.unwrap().value.unwrap();
                    let game = value.as_str().unwrap();
                    let result = news::get_news(game, &self.steam_api_key, &self.client_req).await;
                
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
                command => unreachable!("Unknown command: {}", command),
            };
        // send `response_content` to the discord server
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(response_content))
        })
            .await.expect("Cannot respond to slash command");
    }
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
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

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
