use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct BackupEntity {
    pub path: PathBuf,
    pub recursive: bool,
    pub changes: u32,
    pub timer: u32,
    pub exec: String,
}

