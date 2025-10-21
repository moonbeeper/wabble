use std::{fs::File, io::Write, net::SocketAddr, path::Path};

use smart_default::SmartDefault;

pub mod cli;

#[derive(Debug, serde::Serialize, serde::Deserialize, SmartDefault)]
pub struct HttpSettings {
    #[default(SocketAddr::from(([127, 0, 0, 1], 8080)))]
    pub bind: SocketAddr,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, SmartDefault)]
pub struct LoggingSettings {
    #[default(true)]
    pub enabled: bool,
    #[default("info")]
    pub level: String,
    pub format: LoggingSettingsFormat,
    #[default(true)]
    pub show_file_info: bool,
    #[default(false)]
    pub show_thread_ids: bool,
    #[default(true)]
    pub show_line_numbers: bool,
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug, Default)]
pub enum LoggingSettingsFormat {
    #[default]
    Normal,
    Pretty,
    Compact,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Settings {
    pub http: HttpSettings,
    pub logging: LoggingSettings,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        println!("Loading settings...");
        let config = config::Config::builder()
            .add_source(config::File::with_name("settings.toml").required(false))
            .add_source(config::Environment::with_prefix("WAB"))
            .build()?;

        match config.try_deserialize::<Self>() {
            Ok(settings) => {
                println!("Settings loaded successfully!");
                Ok(settings)
            }
            Err(e) => {
                println!(
                    "Failed to deserialize settings from settings file! Will be using the defaults: {e:?}"
                );
                Ok(Self::default())
            }
        }
    }

    pub fn create_settings_file() -> anyhow::Result<()> {
        let path = Path::new("settings.toml");
        let mut file = File::create(path)?;
        file.write_all(toml::to_string_pretty(&Self::default())?.as_bytes())?;
        Ok(())
    }
}
