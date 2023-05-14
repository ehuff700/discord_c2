use anyhow::__private::kind::TraitKind;
use serde::de::StdError;
use serenity::prelude::SerenityError;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum DiscordC2Error {
    NotFound(String),
    PermissionDenied(String),
    ConfigError(String),
    AgentError(String),
    DiscordError(String),
    NoSessionChannel,
    CommandNotFound(String),
    InvalidShellType,
    StdError(String),
    InvalidInput(String),
    RegexError(String),
    LibraryError(String),
    VarError(String),
    InternalError(String),
}

impl fmt::Display for DiscordC2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiscordC2Error::NotFound(s) => write!(f, "NOT_FOUND: {}", s),
            DiscordC2Error::PermissionDenied(s) => {
                write!(f, "PERMISSION_DENIED: '{}'", s)
            }
            DiscordC2Error::ConfigError(s) => write!(f, "CONFIG_ERROR: {}", s),
            DiscordC2Error::AgentError(s) => write!(f, "AGENT_ERROR: {}", s),
            DiscordC2Error::DiscordError(s) => write!(f, "DISCORD_ERROR: {}", s),
            DiscordC2Error::NoSessionChannel => {
                write!(f, "No session channel was found (that's a problem)")
            }
            DiscordC2Error::CommandNotFound(s) => {
                write!(f, "Command {} wasn't found, that's a problem", s)
            }
            DiscordC2Error::InvalidShellType => {
                write!(f, "INVALID_SHELL_TYPE")
            }
            DiscordC2Error::StdError(s) => {
                write!(f, "Ran into error processing camera feed: {}", s)
            }
            DiscordC2Error::InvalidInput(s) => {
                write!(f, "Invalid input was provided: {}", s)
            }
            DiscordC2Error::RegexError(s) => write!(f, "REGEX_ERROR: {}", s),
            DiscordC2Error::LibraryError(s) => write!(f, "LIBRARY_ERROR: {}", s),
            DiscordC2Error::VarError(s) => write!(f, "VAR_ERROR: {}", s),
            DiscordC2Error::InternalError(s) => write!(f, "{}", s),
        }
    }
}

impl From<std::io::Error> for DiscordC2Error {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => DiscordC2Error::NotFound(error.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                DiscordC2Error::PermissionDenied(error.to_string())
            }
            _ => DiscordC2Error::ConfigError(error.to_string()),
        }
    }
}

impl From<SerenityError> for DiscordC2Error {
    fn from(error: SerenityError) -> Self {
        error.anyhow_kind();
        DiscordC2Error::DiscordError(error.to_string())
    }
}

impl From<Box<dyn StdError>> for DiscordC2Error {
    fn from(error: Box<dyn StdError>) -> Self {
        DiscordC2Error::StdError(error.to_string())
    }
}

impl From<anyhow::Error> for DiscordC2Error {
    fn from(error: anyhow::Error) -> Self {
        error.anyhow_kind();
        DiscordC2Error::DiscordError(error.to_string())
    }
}

impl From<JoinError> for DiscordC2Error {
    fn from(error: JoinError) -> Self {
        error.anyhow_kind();
        DiscordC2Error::LibraryError(error.to_string())
    }
}
impl Error for DiscordC2Error {}
