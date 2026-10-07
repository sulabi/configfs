use configfs::{Config, ConfigPath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppSettings {
    port: u16,
    verbose: bool,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 9000,
            verbose: true,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config: Config<AppSettings> = Config::new(ConfigPath::Custom(dir.path().to_path_buf()))?;
    let settings = config.read_or_default()?;

    if settings.verbose {
        println!("using port: {}", settings.port);
    }

    Ok(())
}
