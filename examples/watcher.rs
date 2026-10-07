use std::path::PathBuf;

use configfs::{Config, ConfigPath};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct AppSettings {
    file_path: PathBuf,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("./test.txt"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config: Config<AppSettings> = Config::new(ConfigPath::Custom(dir.path().to_path_buf()))?;

    let config_file = &config.file.clone();

    let mut shared_config = config.load_shared_or_default()?;

    shared_config.on_reload(|new_conf| {
        println!("file reloaded, new path: {}", new_conf.file_path.display());
    })?;

    let _watcher = shared_config.spawn_watcher()?;

    println!("edit file and see changes at {}", config_file.display());

    tokio::signal::ctrl_c().await?;

    Ok(())
}
