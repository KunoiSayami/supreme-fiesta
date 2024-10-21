mod code;
mod config;
mod platform;

use clap::arg;
use config::Config;

async fn async_main(config_file: &str) -> anyhow::Result<()> {
    let config = Config::load(config_file).await?;

    let bot = platform::bot(&config)?;

    platform::bot_run(bot, config).await?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let matches = clap::command!()
        .args(&[arg!([CONFIG] "Configure file").default_value("config.toml")])
        .get_matches();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(matches.get_one::<String>("CONFIG").unwrap()))
}
