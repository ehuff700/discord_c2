use anyhow::__private::kind::TraitKind;
use serenity::prelude::SerenityError;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use serde::de::StdError;

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
}

impl fmt::Display for DiscordC2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiscordC2Error::NotFound(s) => write!(f, "{} was not found", s),
            DiscordC2Error::PermissionDenied(s) => {
                write!(f, "Permission denied for reason: '{}'", s)
            }
            DiscordC2Error::ConfigError(s) => write!(f, "Config error {}", s),
            DiscordC2Error::AgentError(s) => write!(f, "Agent error {}", s),
            DiscordC2Error::DiscordError(s) => write!(f, "Discord error {}", s),
            DiscordC2Error::NoSessionChannel => {
                write!(f, "No session channel was found (that's a problem)")
            }
            DiscordC2Error::CommandNotFound(s) => {
                write!(f, "Command {} wasn't found, that's a problem", s)
            }
            DiscordC2Error::InvalidShellType => {
                write!(f, "Invalid shell type was provided.")
            }
            DiscordC2Error::StdError(s) => write!(f, "Ran into error processing camera feed: {}", s),
            DiscordC2Error::InvalidInput(s) => {
                write!(f, "Invalid input was provided: {}", s)
            }
            DiscordC2Error::RegexError(s) => write!(f, "Regex error: {}", s),
        }
    }
}

impl From<std::io::Error> for DiscordC2Error {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                DiscordC2Error::NotFound(error.to_string())
            },
            std::io::ErrorKind::PermissionDenied => {
                DiscordC2Error::PermissionDenied(error.to_string())
            }
            _ => DiscordC2Error::ConfigError(error.to_string()),
        }
    }
}

impl From<SerenityError> for DiscordC2Error {
    fn from(error: SerenityError) -> Self {
        match error.anyhow_kind() {
            _ => DiscordC2Error::DiscordError(error.to_string()),
        }
    }
}

impl From <Box<dyn StdError>> for DiscordC2Error {
    fn from(error: Box<dyn StdError>) -> Self {
        DiscordC2Error::StdError(error.to_string())
    }
}

impl From<anyhow::Error> for DiscordC2Error {
    fn from(error: anyhow::Error) -> Self {
        match error.anyhow_kind() {
            _ => DiscordC2Error::DiscordError(error.to_string()),
        }
    }
}
impl Error for DiscordC2Error {}
