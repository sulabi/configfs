use crate::{Config, ConfigError};

#[cfg(feature = "watcher")]
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use serde::{Serialize, de::DeserializeOwned};
#[allow(unused_imports)]
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

#[cfg(feature = "watcher")]
type ReloadCallback<T> = Arc<dyn Fn(&T) + Send + Sync>;

pub struct SharedConfig<T> {
    pub data: Arc<RwLock<T>>,
    pub storage: Arc<Config<T>>,

    #[cfg(feature = "watcher")]
    pub on_reload: Arc<RwLock<Option<ReloadCallback<T>>>>,
}

impl<T> Clone for SharedConfig<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            storage: Arc::clone(&self.storage),

            #[cfg(feature = "watcher")]
            on_reload: Arc::clone(&self.on_reload),
        }
    }
}

impl<T: Serialize + DeserializeOwned> SharedConfig<T> {
    pub fn get<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ConfigError> {
        let guard = self.data.read().map_err(|_| ConfigError::LockPoisoned)?;

        Ok(f(guard.deref()))
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ConfigError> {
        let mut guard = self.data.write().map_err(|_| ConfigError::LockPoisoned)?;
        Ok(f(guard.deref_mut()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let guard = self.data.read().map_err(|_| ConfigError::LockPoisoned)?;

        self.storage.write(guard.deref())
    }

    #[cfg(feature = "watcher")]
    pub fn reload(&self) -> Result<(), ConfigError> {
        let fresh_data: T = self.storage.read()?;
        {
            let mut guard = self.data.write().map_err(|_| ConfigError::LockPoisoned)?;
            *guard = fresh_data;
        }

        let callback = self
            .on_reload
            .write()
            .map_err(|_| ConfigError::LockPoisoned)?
            .clone();

        if let Some(callback) = callback {
            self.get(|d| callback(d))?;
        }

        Ok(())
    }

    #[cfg(feature = "watcher")]
    pub fn on_reload(&mut self, f: impl Fn(&T) + Send + Sync + 'static) -> Result<(), ConfigError> {
        let mut guard = self
            .on_reload
            .write()
            .map_err(|_| ConfigError::LockPoisoned)?;
        *guard = Some(Arc::new(f));
        Ok(())
    }
}

#[cfg(feature = "watcher")]
impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> SharedConfig<T> {
    pub fn spawn_watcher(&self) -> Result<RecommendedWatcher, ConfigError> {
        let path = &self.storage.file;
        let shared = self.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res
                && matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Metadata(_)) | EventKind::Create(_)
                )
            // ModifyKind::Data fires twice sometimes, so i'll track with ModifyKind::Metadata
            // instead
            {
                let _ = shared.reload();
            }
        })
        .map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: io::Error::other(e.to_string()),
        })?;

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::Io {
                path: path.to_path_buf(),
                source: io::Error::other(e.to_string()),
            })?;

        Ok(watcher)
    }
}
