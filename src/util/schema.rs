#[derive(Debug, Default)]
pub struct UploadForm {
    pub action: Action,
    pub content: String,
    pub target_file_path: String,
}

#[derive(Debug)]
pub enum Action {
    Safe,
    Force,
}

impl Action {
    pub fn from_str(action: &str) -> Self {
        match action {
            "safe" => Action::Safe,
            "force" => Action::Force,
            _ => Action::Safe,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Safe => "safe".to_string(),
            Self::Force => "force".to_string(),
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Safe
    }
}
