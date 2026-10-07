# configfs

[![Build Status](https://github.com/sulabi/configfs/actions/workflows/rust.yml/badge.svg)](https://github.com/sulabi/configfs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/configfs.svg)](https://crates.io/crates/configfs)
[![Documentation](https://docs.rs/configfs/badge.svg)](https://docs.rs/configfs)
[![License](https://img.shields.io/crates/l/configfs.svg)](https://crates.io/crates/configfs)

A small lightweight filesystem config manager
`configfs` provides an api to load, deserialize and write
config files for your application.
This currently only supports the TOML format, however I shall introduce more formats in the future.

## Installation

Add `configfs` and `serde` to your `Cargo.toml`
```toml
[dependencies]
configfs = "0.2.2"
serde = { version = "1", features = ["derive"] }
```

## Usage

### Reading Config

```rust
use configfs::{Config, ConfigPath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppSettings {
    port: u16,
    verbose: bool
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 9000,
            verbose: true
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigPath::Custom("./appconf".into()))?;
    let settings = config.read_or_default::<AppSettings>()?;

    if settings.verbose {
        println!("using port: {}", settings.port);
    }

    Ok(())
}
```

### Writing Config

```rust
use configfs::{Config, ConfigPath};
use serde::Serialize;

#[derive(Serialize)]
struct AppSettings {
    username: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigPath::Custom("./appconf".into()))?;
    let settings = AppSettings {
        username: "jimmy".into()
    };

    config.write(&settings)?;

    Ok(())
}
```

## Features

### `system-dirs`
Enables support to access the users local config directory, this is fetched using the `dirs` crate.

```toml
[dependencies]
configfs = { version = "0.1", features = ["system-dirs"] }
```

An example that will save the config folder `app` in `~/.config/`.

```rust
let config = Config::new(ConfigPath::System("app"))?;
```

### `watcher`
Enables support to watch files as they reload.

An example of this to see file reloading can be seen in
[`examples/watcher.rs`](examples/watcher.rs).

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0
