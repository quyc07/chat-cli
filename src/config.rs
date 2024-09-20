use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Host {
    pub(crate) server: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) host: Host,
}

impl Settings {
    pub fn new(build_mode: &str) -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(config::File::with_name(&format!("config/{}", build_mode)))
            .build()?
            .try_deserialize()
    }
}