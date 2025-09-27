use anyhow::Result;
use poise::serenity_prelude::{self as serenity};
use sqlx::SqlitePool;

struct Data {
    db: SqlitePool,
}
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[poise::command(slash_command, prefix_command)]
async fn himalahiafy(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[description = "Himalahia level"] level: u32,
) -> Result<()> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let word_count = himalahia_level_to_word_count(level);

    sqlx::query("INSERT INTO word_limit (user_id, word_limit) VALUES (?, ?)")
        .bind(u.id.get() as i64)
        .bind(word_count)
        .execute(&ctx.data().db)
        .await?;

    let response = format!("{} now is himalahia level 🏔️{}!", u.name, level);

    ctx.reply(response).await?;

    Ok(())
}

fn himalahia_level_to_word_count(level: u32) -> u32 {
    // x -> 0 = 1024; x -> ∞ = 3
    // f(x)\=\frac{1021}{\left(x+1\right)^{2}}+3
    1021 / ((level + 1).pow(2)) + 3
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all();

    let db = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!().run(&db).await?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![himalahiafy()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    client.start().await?;

    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    data: &Data,
) -> Result<()> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            let words = new_message.content.split_whitespace().count();

            if words <= 3 {
                return Ok(());
            }

            let user = new_message.author.id.get();
            let limit: Option<u32> =
                sqlx::query_scalar("SELECT word_limit FROM word_limit WHERE user_id = ?")
                    .bind(user as i64)
                    .fetch_optional(&data.db)
                    .await?;

            if let Some(limit) = limit {
                if words as u32 > limit {
                    new_message.delete(&ctx.http).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
