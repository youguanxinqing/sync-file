use std::{fmt::Display, str::FromStr};

#[derive(Debug, Default)]
pub struct UploadForm {
    pub action: Action,
    pub content: String,
    pub target_file_path: String,
}

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    Safe,
    Force,
}

#[derive(Debug)]
pub struct NotActionError;

impl Display for NotActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "not support action error".fmt(f)
    }
}

impl FromStr for Action {
    type Err = NotActionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let action = match s {
            "safe" => Action::Safe,
            "force" => Action::Force,
            _ => Action::Safe,
        };
        Ok(action)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => f.write_str("safe"),
            Self::Force => f.write_str("force"),
        }
    }
}
