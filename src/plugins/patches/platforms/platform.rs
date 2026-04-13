use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Steam,
    Riot,
    Unknown,
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

impl From<&str> for Platform {
    fn from(s: &str) -> Self {
        match s {
            "steam" => Platform::Steam,
            "riot" => Platform::Riot,
            _ => Platform::Unknown,
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::Steam => write!(f, "steam"),
            Platform::Riot => write!(f, "riot"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(input: &str) -> Result<Platform, Self::Err> {
        match input {
            "steam" => Ok(Platform::Steam),
            "riot" => Ok(Platform::Riot),
            _ => Ok(Platform::Unknown),
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Unknown
    }
}
