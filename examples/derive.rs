use configfs::{Config, ConfigError, ConfigFile};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Config)]
#[config(path = "target/example/derive/config.toml")]
struct AppSettings {
    username: String,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            username: "jimmy".into(),
        }
    }
}

fn main() -> Result<(), ConfigError> {
    AppSettings::init_or_default()?;

    println!("username: {}", AppSettings::get().username);

    AppSettings::update(|s| s.username = "cool".into())?;

    {
        let mut s = AppSettings::get_mut();
        s.username.push('!');
        s.write()?;
    }

    println!("new username: {}", AppSettings::get().username);

    Ok(())
}
