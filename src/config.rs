use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;

pub fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Trace)
        .init();
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub baseline_sites: Vec<String>,
}

impl Config {
    fn get_config_path() -> String {
        env::var("MANDOWN_CONFIG").unwrap_or_else(|_| "config.yaml".to_string())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let contents = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
