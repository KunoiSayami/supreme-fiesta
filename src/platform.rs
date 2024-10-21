use std::sync::{Arc, LazyLock};

use chrono::DateTime;
use log::warn;
use tap::TapFallible;
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{dialogue::GetChatId, Dispatcher, HandlerExt, UpdateFilterExt},
    macros::BotCommands,
    payloads::SendPhotoSetters,
    prelude::dptree,
    requests::{Requester, RequesterExt},
    types::{ChatId, InputFile, Message, ParseMode, Update},
    Bot,
};

use crate::{
    code::{into_barcode, merge2memory},
    config::Config,
};

pub static TELEGRAM_ESCAPE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"([_*\[\]\(\)~`>#\+-=|\{}\.!])").unwrap());
pub static TEXT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[\w\d]{5,}").unwrap());

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

pub async fn bot_run(bot: BotType, config: Config) -> anyhow::Result<()> {
    let owner = config.platform().owner();
    let self_id = Arc::new(config.barcode_id());

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
                .filter(move |msg: Message| {
                    msg.chat_id().eq(&Some(ChatId(owner)))
                        && msg.text().is_some_and(|s| !s.starts_with('/'))
                })
                .endpoint(
                    |msg: Message, bot: BotType, self_id: Arc<String>| async move {
                        handle_message(bot, msg, self_id).await
                    },
                ),
        );

    let dispatcher = Dispatcher::builder(bot, dptree::entry().branch(handle_message))
        .dependencies(dptree::deps![self_id])
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

pub async fn handle_message(
    bot: BotType,
    msg: Message,
    self_id: Arc<String>,
) -> anyhow::Result<()> {
    let text = msg.text().unwrap();
    if !TEXT_RE.is_match(text) {
        warn!("Ignore wrong input {text}");
        return Ok(());
    }

    let ret = match tokio::task::spawn_blocking({
        let barcode = into_barcode(text);
        move || merge2memory(self_id.clone(), &barcode)
    })
    .await?
    {
        Ok(image) => image,
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Encode error: {e:?}"))
                .await?;
            return Ok(());
        }
    };
    bot.send_photo(msg.chat.id, InputFile::memory(ret))
        .caption(current_time().tg_str())
        .await?;

    Ok(())
}
fn current_time() -> String {
    let time: DateTime<chrono::prelude::Local> =
        DateTime::from_timestamp(kstool::time::get_current_second() as i64, 0)
            .unwrap()
            .into();
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
