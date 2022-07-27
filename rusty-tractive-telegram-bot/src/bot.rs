use std::borrow::Cow;

use anyhow::Result;
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::{Data, Json, TypedHeader};
use poem::{handler, post, EndpointExt, Route, Server};
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::headers::SecretToken;
use rusty_shared_telegram::methods::Method;
use rusty_shared_telegram::{methods, models};
use secstr::SecUtf8;
use tracing::{debug, info, instrument, warn};

use crate::middleware::TracingMiddleware;

pub async fn run(
    api: BotApi,
    bind_endpoint: String,
    webhook_url: String,
    secret_token: SecUtf8,
) -> Result<()> {
    info!("setting up the bot…");
    methods::SetMyCommands::default()
        .command(models::BotCommand {
            command: Cow::Borrowed("start"),
            description: Cow::Borrowed("Tells your chat ID"),
        })
        .call(&api)
        .await?;
    methods::SetWebhook::new(webhook_url)
        .allow_update(methods::AllowedUpdate::Message)
        .secret_token(secret_token.unsecure())
        .call(&api)
        .await?;

    info!("running the bot…");
    let app = Route::new()
        .at("/", post(on_update))
        .with(AddData::new(api))
        .with(AddData::new(SecretToken(secret_token)))
        .with(TracingMiddleware);
    Server::new(TcpListener::bind(bind_endpoint))
        .run(app)
        .await?;
    Ok(())
}

#[handler]
#[instrument(skip_all, fields(update.id = update.id))]
async fn on_update(
    TypedHeader(SecretToken(secret_token)): TypedHeader<SecretToken>,
    Json(update): Json<models::Update>,
    bot_api: Data<&BotApi>,
    expected_secret_token: Data<&SecretToken>,
) -> Result<StatusCode> {
    info!("👌 handling the update…");

    if !expected_secret_token.is_valid(&secret_token) {
        warn!("secret token mismatch");
        return Ok(StatusCode::UNAUTHORIZED);
    }

    match update.payload {
        models::UpdatePayload::Message(message) => match message.text {
            Some(text) if text.starts_with("/start") => {
                methods::SendMessage::new(
                    message.chat.id,
                    format!(r#"👋 Your chat ID is `{}`\."#, message.chat.id),
                )
                .parse_mode(models::ParseMode::MarkdownV2)
                .reply_to_message_id(message.id)
                .call(&bot_api)
                .await?;
            }
            _ => {
                debug!(?message.text, "ignoring the unsupported message");
            }
        },
        payload => {
            debug!(?payload, "ignoring the unsupported update");
        }
    }

    info!("👍 handled");
    Ok(StatusCode::NO_CONTENT)
}
