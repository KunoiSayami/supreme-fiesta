use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use chrono::DateTime;
use log::warn;
use tap::TapFallible;
use teloxide::{
    Bot,
    adaptors::DefaultParseMode,
    dispatching::{Dispatcher, HandlerExt, UpdateFilterExt},
    macros::BotCommands,
    payloads::SendPhotoSetters,
    prelude::dptree,
    requests::{Requester, RequesterExt},
    types::{InputFile, Message, ParseMode, Update},
};

use crate::{
    code::{into_barcode, merge2memory, qr_memory, single_memory},
    config::Config,
};

pub static TELEGRAM_ESCAPE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"([_*\[\]\(\)~`>#\+-=|\{}\.!])").unwrap());
pub static TEXT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[\w\d]{5,}").unwrap());
pub static ALL_NUMERIC_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^\d+$").unwrap());

trait IntoTelegramString: AsRef<str> {
    fn tg_str(&self) -> String {
        TELEGRAM_ESCAPE_RE.replace_all(self.as_ref(), "\\$1").into()
    }
}

impl IntoTelegramString for String {}
impl IntoTelegramString for &str {}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Ping,
}

pub fn bot(config: &Config) -> anyhow::Result<BotType> {
    let bot = Bot::new(config.platform().api_key());
    Ok(match config.platform().server() {
        Some(url) => bot.set_api_url(url.parse()?),
        None => bot,
    }
    .parse_mode(ParseMode::MarkdownV2))
}

pub type BotType = DefaultParseMode<Bot>;

pub type UserMap = Arc<HashMap<i64, String>>;

pub async fn bot_run(bot: BotType, config: Config) -> anyhow::Result<()> {
    let user_map: UserMap = Arc::new(config.user_entries().collect());

    let handle_message = Update::filter_message()
        .branch(
            dptree::entry()
                .filter(|msg: Message| msg.chat.is_private())
                .filter_command::<Command>()
                .endpoint(|msg: Message, bot: BotType, cmd: Command| async move {
                    match cmd {
                        Command::Ping => handle_ping(bot, msg).await,
                    }
                    .tap_err(|e| log::error!("Handle command error: {e:?}"))
                }),
        )
        .branch(
            dptree::entry()
                .filter(|msg: Message| {
                    msg.chat.is_private() && msg.text().is_some_and(|s| !s.starts_with('/'))
                })
                .endpoint(|msg: Message, bot: BotType, user_map: UserMap| async move {
                    handle_message(bot, msg, user_map).await
                }),
        );

    let dispatcher = Dispatcher::builder(bot, dptree::entry().branch(handle_message))
        .dependencies(dptree::deps![user_map])
        .default_handler(|_| async {});

    #[cfg(not(debug_assertions))]
    dispatcher.enable_ctrlc_handler().build().dispatch().await;

    #[cfg(debug_assertions)]
    tokio::select! {
        _ = async move {
            dispatcher.build().dispatch().await
        } => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    Ok(())
}

pub async fn handle_ping(bot: BotType, msg: Message) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        format!(
            "Chat id: `{id}`\nVersion: {version}",
            id = msg.chat.id.0,
            version = env!("CARGO_PKG_VERSION").tg_str()
        ),
    )
    .await?;
    Ok(())
}

pub async fn handle_message(bot: BotType, msg: Message, user_map: UserMap) -> anyhow::Result<()> {
    let text = msg.text().unwrap();
    if !TEXT_RE.is_match(text) {
        warn!("Ignore wrong input {text}");
        return Ok(());
    }

    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64);
    let barcode_id = user_id.and_then(|id| user_map.get(&id).cloned());
    let is_numeric = ALL_NUMERIC_RE.is_match(text);

    let text = text.to_owned();
    let ret = tokio::task::spawn_blocking(move || {
        if is_numeric {
            match barcode_id {
                Some(id) => merge2memory(Arc::new(id), &into_barcode(&text)),
                None => single_memory(&into_barcode(&text)),
            }
        } else {
            qr_memory(&text)
        }
    })
    .await?;

    send_image(&bot, &msg, ret).await
}

async fn send_image(
    bot: &BotType,
    msg: &Message,
    result: anyhow::Result<Vec<u8>>,
) -> anyhow::Result<()> {
    match result {
        Ok(image) => {
            bot.send_photo(msg.chat.id, InputFile::memory(image))
                .caption(current_time().tg_str())
                .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Encode error: {e:?}"))
                .await?;
        }
    }
    Ok(())
}

fn current_time() -> String {
    let time: DateTime<chrono::prelude::Local> =
        DateTime::from_timestamp(kstool::time::get_current_second() as i64, 0)
            .unwrap()
            .into();
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
