mod ping_command;

use futures::{stream::StreamExt};
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::{InteractionData::ApplicationCommand},
    gateway::{Intents, payload::incoming::InteractionCreate},
    id::{marker::ApplicationMarker, Id},
};

pub struct Ctx<'a> { 
    i: &'a Box<InteractionCreate>,
    http: Arc<HttpClient>,
    id: Id<ApplicationMarker>
}

pub type CommandReturn = Result<(), Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token =
        env::var("DISCORD_TOKEN").expect("Could not find enviroment variable 'DISCORD_TOKEN'");

    // Use intents to only receive guild message events.

    // A cluster is a manager for multiple shards that by default
    // creates as many shards as Discord recommends.
    let (cluster, mut events) = Cluster::new(
        token.to_owned(),
        Intents::GUILD_MESSAGES | Intents::GUILD_MESSAGE_REACTIONS | Intents::MESSAGE_CONTENT,
    )
    .await?;
    let cluster = Arc::new(cluster);

    // Start up the cluster.
    let cluster_spawn = Arc::clone(&cluster);

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = Arc::new(HttpClient::new(token));

    let application_id = {
        let response = http.current_user_application().exec().await?;

        response.model().await?.id
    };

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    


    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(
            shard_id,
            event,
            Arc::clone(&http),
            application_id.clone()
        ));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            if msg.content == "!ping" {
                http.create_message(msg.channel_id)
                    .content("Pong!")?
                    .exec()
                    .await?;
            }

            println!("{}", msg.content)
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {shard_id}");
        }
        Event::InteractionCreate(i) => match &i.data {
            Some(ApplicationCommand(data)) => {
                
                let payload = Ctx {
                    http,
                    i: &i,
                    id: application_id
                };

                if data.name == "ping" {
                    ping_command::handle_event(payload).await?;
                }
            }
            _ => {}
        },
        // Other events here...
        _ => {}
    }

    Ok(())
}
