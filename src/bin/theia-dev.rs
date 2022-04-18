use ::color_eyre::eyre::Result;
use ::std::env;
use ::std::path::PathBuf;
use ::theia::error::TheiaCliError;
use ::theia::prelude::*;

fn main_plugin_validate<'a>(cmd: &'a CommandInvocation) -> Result<()> {
    if cmd.help_requested() || cmd.arguments.len() < 1 {
        println!(
            "Usage: theia-dev {} [OPTIONS] <path to plugin dir>",
            cmd.command
        );
        println!("Options:");
        println!("    -help: this help");
        println!("    -p -printdbg: debug-print the parsed plugin config");

        return Ok(());
    }

    let path = PathBuf::from(&cmd.arguments[0]);
    let plugin = TheiaPlugin::new(&path)?;

    if cmd.has_startflag_any(&["-p", "-printdbg"]) {
        println!("{:#?}", plugin.config);
    }

    Ok(())
}

fn main_help<'a>(_cmd: &'a CommandInvocation) -> Result<()> {
    println!("theia-dev (from theia v{})", env!("CARGO_PKG_VERSION"));
    println!("Available subcommands: ");
    println!("    help: this subcommand list");
    println!("    cmd-parser-dump: dump parsed command");
    println!("    plugin-validate: parse a plugin's configuration");

    Ok(())
}

fn main() -> Result<()> {
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
            "cmd-parser-dump" => {
                println!("{:#?}", cmd);
                Ok(())
            }
            "plugin-validate" => main_plugin_validate(&cmd),
            _ => Err(TheiaCliError::UnknownSubcommand(cmd.command))?,
        }
    } else {
        Err(TheiaCliError::NoSubcommand)?
    }
}
