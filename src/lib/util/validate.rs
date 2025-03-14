use std::fs;

pub fn verify_file_existed(file: &str) -> anyhow::Result<String> {
    if file == "..." {
        return Ok(file.to_string());
    }
    
    fs::exists(file)
        .map_err(|e| anyhow::Error::from(e))
        .map(|_| file.to_string())
}
