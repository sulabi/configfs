//! # configfs
//! A small lightweight filesystem config manager
//! `configfs` provides an api to load, deserialize and write
//! config files for your application.
//! This currently only supports the TOML format, however I shall
//! introduce more formats in the future.
//!
//! ## Usage
//!
//! ```rust
//! use configfs::{Config, ConfigDirectory};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct AppSettings {
//!     port: u16,
//!     verbose: bool
//! }
//! impl Default for AppSettings {
//!     fn default() -> Self {
//!         Self {
//!             port: 9000,
//!             verbose: true
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let dir = tempfile::tempdir()?;
//!     let config: Config<AppSettings> = Config::new(ConfigDirectory::Custom(dir.path().to_path_buf()))?;
//!     let settings = config.read_or_default()?;
//!
//!     if settings.verbose {
//!         println!("using port: {}", settings.port);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Writing Config
//!
//! ```rust
//! use configfs::{Config, ConfigDirectory};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct AppSettings {
//!     username: String
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let dir = tempfile::tempdir()?;
//!     let config: Config<AppSettings> = Config::new(ConfigDirectory::Custom(dir.path().to_path_buf()))?;
//!     let settings = AppSettings {
//!         username: "jimmy".into()
//!     };
//!
//!     config.write(&settings)?;
//!
//!     Ok(())
//! }
//! ```

use std::{
    fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

pub use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod error;
mod shared;

pub use error::*;
pub use shared::*;

/// Target directory of the configuration files
pub enum ConfigDirectory {
    /// System default configuration directory path (`~/.config/app_name`)
    /// Can also end with a file path. e.g:
    /// `ConfigDirectory::System("app_name/this.toml")`
    #[cfg(feature = "system-dirs")]
    System(&'static str),

    /// Custom file path
    Custom(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Config<T> {
    /// Filename of the current configuration file (default `config.toml`)
    pub file: PathBuf,
    _marker: PhantomData<T>,
}

impl<T> Config<T> {
    pub fn new(dir: ConfigDirectory) -> Result<Self, ConfigError> {
        let (config_path, like_dir) = match dir {
            #[cfg(feature = "system-dirs")]
            ConfigDirectory::System(app_name) => {
                let config_dir = dirs::config_dir().ok_or(ConfigError::SystemConfigNotFound)?;
                let path = config_dir.join(app_name);
                let is_dir = Config::like_dir(&path);

                (path, is_dir)
            }

            ConfigDirectory::Custom(config_path) => {
                let is_dir = Config::like_dir(&config_path);
                (config_path, is_dir)
            }
        };

        let config_file = if like_dir {
            config_path.join("config.toml")
        } else {
            config_path
        };

        Ok(Self {
            file: config_file,
            _marker: PhantomData,
        })
    }

    /// Returns the parent of the file
    pub fn parent(&self) -> Result<&Path, ConfigError> {
        self.file.parent().ok_or(ConfigError::SystemConfigNotFound)
    }

    /// Changes the current configuration file
    pub fn set_file<C>(self, file: impl Into<PathBuf>) -> Config<C> {
        Config {
            file: file.into(),
            _marker: PhantomData,
        }
    }
}

impl Config<()> {
    fn like_dir(p: &Path) -> bool {
        if p.exists() {
            return p.is_dir();
        }
        let ending_sep = p
            .as_os_str()
            .to_string_lossy()
            .ends_with(std::path::is_separator);
        let no_ext = p.extension().is_none();

        ending_sep || no_ext
    }
}

impl<T: DeserializeOwned> Config<T> {
    /// Reads and deserializes the configuration file
    pub fn read(&self) -> Result<T, ConfigError> {
        let content = fs::read_to_string(&self.file).map_err(|e| ConfigError::Io {
            path: self.file.clone(),
            source: e,
        })?;

        Ok(toml::from_str::<T>(&content)?)
    }
}

impl<T: Serialize + DeserializeOwned + Default> Config<T> {
    /// Reads and deserializes the configuration file. If missing config is written
    pub fn read_or_default(&self) -> Result<T, ConfigError> {
        match self.read() {
            Ok(data) => Ok(data),
            Err(ConfigError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                let default_conf = T::default();
                self.write(&default_conf)?;
                Ok(default_conf)
            }
            Err(err) => Err(err),
        }
    }
}

impl<T: Serialize> Config<T> {
    /// Serializes and writes config to disk as pretty TOML file.
    pub fn write(&self, data: &T) -> Result<(), ConfigError> {
        if let Some(parent) = &self.file.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|err| ConfigError::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
        }

        let content = toml::to_string_pretty(data)?;
        fs::write(&self.file, content).map_err(|err| ConfigError::Io {
            path: self.file.clone(),
            source: err,
        })?;

        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned> Config<T> {
    /// Loads the config and stores it in a thread safe `SharedConfig`
    pub fn load_shared(self) -> Result<SharedConfig<T>, ConfigError> {
        let data = self.read()?;
        Ok(SharedConfig {
            data: Arc::new(RwLock::new(data)),
            storage: Arc::new(self),

            #[cfg(feature = "watcher")]
            on_reload: Arc::new(RwLock::new(None)),
        })
    }
}
impl<T: Serialize + DeserializeOwned + Default> Config<T> {
    /// Loads the config, writes and returns `T::default()` if missing, and stores it in a thread safe `SharedConfig`
    pub fn load_shared_or_default(self) -> Result<SharedConfig<T>, ConfigError> {
        let data = match self.read() {
            Ok(data) => data,
            Err(ConfigError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                let default_conf = T::default();
                self.write(&default_conf)?;
                default_conf
            }
            Err(err) => return Err(err),
        };

        Ok(SharedConfig {
            data: Arc::new(RwLock::new(data)),
            storage: Arc::new(self),

            #[cfg(feature = "watcher")]
            on_reload: Arc::new(RwLock::new(None)),
        })
    }
}
