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
    pub(crate) timeout: u64,
}

#[derive(Default, Deserialize, Clone, Debug)]
pub(crate) struct Epix {
    pub(crate) base_url: String,
    pub(crate) domain: Domain,
    pub(crate) identifier_domain: String,
    pub(crate) data_source: String,
}

#[derive(Default, Deserialize, Clone, Debug)]
pub(crate) struct Domain {
    pub(crate) name: String,
    pub(crate) description: String,
}

#[derive(Default, Deserialize, Clone, Debug)]
pub(crate) struct Gpas {
    pub(crate) base_url: String,
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
