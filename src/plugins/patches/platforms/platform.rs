use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Steam,
    Riot,
    Unknown,
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
