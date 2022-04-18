use ::color_eyre::eyre::Result;
use ::std::env;
use ::std::path::PathBuf;
use ::theia::error::TheiaCliError;
use ::theia::prelude::*;

async fn run<'a>(cmd: &'a CommandInvocation) -> Result<()> {
    if cmd.help_requested() {
        println!("Usage: theia {} [OPTS] [<path to theia.toml>]", cmd.command);
        return Ok(());
    }

    if cmd.has_startflag_any(&["-de", "-dotenv"]) {
        if let Err(err) = ::dotenv::dotenv() {
            warn!("Loading `.env` failed: {}", err);
        }
    }

    let config_path = if cmd.arguments.len() < 1 {
        PathBuf::from("theia.toml")
    } else {
        PathBuf::from(&cmd.arguments[0])
    };

    info!("Creating bot with config_path={:?}", config_path);
    let mut bot = Theia::new(&config_path)?;
    bot.reload().await?;

    info!("Starting bot...");
    bot.run().await?;

    Ok(())
}

fn main_help<'a>(_cmd: &'a CommandInvocation) -> Result<()> {
    println!("Available subcommands: ");
    println!("    help: this subcommand list");
    println!("    run: run the bot");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_SPANTRACE").is_err() {
        env::set_var("RUST_SPANTRACE", "0");
    }

    color_eyre::install()?;
    if env::var("RUST_LOG").is_ok() {
        ::tracing_subscriber::fmt()
            .with_writer(::std::io::stderr)
            .init();
    }

    if let Some(cmd) = CommandInvocation::parse(
        &[""],
        &env::args()
            .skip(1)
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join(" "),
    ) {
        match cmd.command.as_ref() {
            "help" => main_help(&cmd),
            "run" => run(&cmd).await,
            _ => Err(TheiaCliError::UnknownSubcommand(cmd.command))?,
        }
    } else {
        Err(TheiaCliError::NoSubcommand)?
    }
}
