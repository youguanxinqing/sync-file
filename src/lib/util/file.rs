use std::{fs, path::Path};

pub fn create_and_write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> anyhow::Result<()> {
    let dir = path.as_ref().parent();
    if dir.is_some() && !dir.unwrap().exists() {
        fs::create_dir_all(dir.unwrap())?;
    }

    fs::write(path, contents)?;
    Ok(())
}
