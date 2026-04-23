use crate::{plugin::PluginError, Error};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    /// Steam platform
    Steam,
    /// Riot platform
    Riot,
}

impl Platform {
    /// Checks if the provided platform string is a valid platform.
    ///
    /// # Arguments
    /// * `platform` - A string slice representing the platform to check.
    ///
    /// # Returns
    /// * `true` if the platform is valid (steam or riot), otherwise `false`.
    pub fn is_valid_platform(platform: &str) -> bool {
        matches!(platform, "steam" | "riot")
    }
}

impl TryFrom<&str> for Platform {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        match s {
            "steam" => Ok(Platform::Steam),
            "riot" => Ok(Platform::Riot),
            _ => Err(Error::Plugin(PluginError::InvalidGamePlatform)),
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::Steam => write!(f, "steam"),
            Platform::Riot => write!(f, "riot"),
        }
    }
}

impl FromStr for Platform {
    type Err = Error;

    fn from_str(input: &str) -> Result<Platform, Self::Err> {
        match input {
            "steam" => Ok(Platform::Steam),
            "riot" => Ok(Platform::Riot),
            _ => Err(Error::Plugin(PluginError::InvalidGamePlatform)),
        }
    }
}
