
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
            "fore" => Action::Force,
            _ => Action::Safe,
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Safe
    }
}
