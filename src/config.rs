use serde::Deserialize;
use std::{fs, sync::OnceLock};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub settings: Settings,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub server_url: String,
    pub org: String,
    pub project: String,
    pub pat_token: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Skal kaldes én gang fra main()
pub fn init_config() {
    let content = fs::read_to_string("config.toml").expect("Kunne ikke læse config.toml");
    let parsed: Config = toml::from_str(&content).expect("Ugyldig config.toml");
    CONFIG.set(parsed).expect("Config allerede sat");
}

/// Global adgang til konfiguration
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Konfiguration er ikke initialiseret endnu")
}