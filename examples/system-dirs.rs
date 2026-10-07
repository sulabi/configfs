use configfs::{Config, ConfigPath};
use serde::Serialize;

#[derive(Serialize)]
struct AppSettings {
    username: String,
}

// The following code will write the config into ~/.config/app/config.toml
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigPath::System("app/config.toml"))?;
    let settings = AppSettings {
        username: "jimmy".into(),
    };

    config.write(&settings)?;

    Ok(())
}
