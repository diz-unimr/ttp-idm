use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub(crate) log_level: String,
    pub(crate) auth: Option<Auth>,
    pub(crate) ttp: Ttp,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct Auth {
    pub(crate) basic: Option<BasicAuth>,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct BasicAuth {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct Ttp {
    pub(crate) epix: Epix,
    pub(crate) gpas: Gpas,
    pub(crate) auth: Option<Auth>,
    pub(crate) retry: Retry,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct Epix {
    pub(crate) base_url: String,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct Gpas {
    pub(crate) base_url: String,
}

#[derive(Default, Debug, Deserialize, Clone)]
pub(crate) struct Retry {
    pub(crate) count: u32,
    pub(crate) timeout: u64,
    pub(crate) wait: u64,
    pub(crate) max_wait: u64,
}

impl AppConfig {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        Config::builder()
            // default config from file
            .add_source(File::with_name("app.yaml"))
            // override values from environment variables
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
