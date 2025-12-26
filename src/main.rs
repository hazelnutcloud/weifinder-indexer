use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let settings_path: PathBuf = env::args()
        .nth(1)
        .or(env::var("INDEXER_SETTINGS_PATH").ok())
        .map(|path_str| path_str.into())
        .or(Some(PathBuf::from("settings.toml")))
        .unwrap();
}
