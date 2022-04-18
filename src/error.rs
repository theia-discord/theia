//! Error types.

use ::displaydoc::Display;
use ::std::error::Error;

/// The core Theia error type.
#[derive(Display, Debug)]
#[non_exhaustive]
pub enum TheiaError {
    /// Configuration file not found
    ConfigNotFound,

    /// Configuration failed to parse: {0}
    ConfigParseError(::toml::de::Error),

    /// No Discord bot token was provided in the `DISCORD_TOKEN` environment variable
    NoDiscordToken,

    /// The plugin at `{0}` failed to load: {1}
    PluginLoad(String, TheiaPluginLoadError),

    /// The plugin `{0}` failed to run: {1}
    PluginRun(String, TheiaPluginRunError),

    /// Serenity error: {0}
    SerenityError(::serenity::Error),

    /// serde_json error: {0}
    SerdeJsonError(::serde_json::Error),

    /// IO error: {0}
    IoError(::std::io::Error),

    /// Utf8Error: {0:?}
    Utf8Error(::std::str::Utf8Error),

    /// FromUtf8Error: {0:?}
    FromUtf8Error(::std::string::FromUtf8Error),

    /// ParseIntError: {0}
    ParseIntError(::std::num::ParseIntError),

    /// An unknown error occurred
    UnknownError,
}

impl Error for TheiaError {}

impl From<::serenity::Error> for TheiaError {
    fn from(se: ::serenity::Error) -> TheiaError {
        TheiaError::SerenityError(se)
    }
}

impl From<::serde_json::Error> for TheiaError {
    fn from(se: ::serde_json::Error) -> TheiaError {
        TheiaError::SerdeJsonError(se)
    }
}

impl From<::std::io::Error> for TheiaError {
    fn from(se: ::std::io::Error) -> TheiaError {
        TheiaError::IoError(se)
    }
}

impl From<::std::str::Utf8Error> for TheiaError {
    fn from(se: ::std::str::Utf8Error) -> TheiaError {
        TheiaError::Utf8Error(se)
    }
}

impl From<::std::string::FromUtf8Error> for TheiaError {
    fn from(se: ::std::string::FromUtf8Error) -> TheiaError {
        TheiaError::FromUtf8Error(se)
    }
}

impl From<::std::num::ParseIntError> for TheiaError {
    fn from(se: ::std::num::ParseIntError) -> TheiaError {
        TheiaError::ParseIntError(se)
    }
}

/// Plugin load errors.
#[derive(Display, Debug, PartialEq)]
#[non_exhaustive]
pub enum TheiaPluginLoadError {
    /// Plugin path does not exist or does not contain a plugin config
    NotFound,

    /// Configuration failed to parse: {0}
    ConfigParseError(::toml::de::Error),
}

impl Error for TheiaPluginLoadError {}

/// Plugin run errors.
#[derive(Display, Debug, PartialEq)]
#[non_exhaustive]
pub enum TheiaPluginRunError {
    /// Plugin existed with non-zero status: `{0:?}`
    ExitStatus(i32),

    /// Plugin was terminated by signal
    Terminated,
}

impl Error for TheiaPluginRunError {}

/// CLI errors.
#[derive(Display, Debug, PartialEq)]
#[non_exhaustive]
pub enum TheiaCliError {
    /// No subcommand specified.
    NoSubcommand,

    /// Unknown subcommand: `{0}`
    UnknownSubcommand(String),
}

impl Error for TheiaCliError {}
