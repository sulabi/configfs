use configfs::{Config, ConfigPath};
use serde::Serialize;

#[derive(Serialize)]
struct AppSettings {
    username: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config = Config::new(ConfigPath::Custom(dir.path().to_path_buf()))?;
    let settings = AppSettings {
        username: "jimmy".into(),
    };

    config.write(&settings)?;

    Ok(())
}
