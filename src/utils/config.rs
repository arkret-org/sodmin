use std::sync::Mutex;
use std::sync::OnceLock;

use crate::utils::storage;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub restrict_base_url: Option<Vec<String>>,
    pub cors_credentials: String,
    pub external_auth_provider: bool,
}

static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();

fn config_mutex() -> &'static Mutex<Config> {
    CONFIG.get_or_init(|| Mutex::new(Config::default()))
}

pub fn get_config() -> Config {
    config_mutex().lock().unwrap().clone()
}

pub fn set_config(config: Config) {
    *config_mutex().lock().unwrap() = config;
}

pub fn clear_config() {
    *config_mutex().lock().unwrap() = Config::default();
    storage::clear();
}

pub fn set_external_auth_provider(value: bool) {
    config_mutex().lock().unwrap().external_auth_provider = value;
}

/// Shape of `/config.json` served by the deployment. Only the coauth
/// public URL is currently consumed; `pages::login` writes it into
/// storage on app boot so `utils::session::coauth_public_url` can read
/// it back from any layer.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct RuntimeConfig {
    #[serde(default, alias = "pasion_public_url")]
    pub coauth_public_url: String,
}

pub async fn load_runtime_config() -> RuntimeConfig {
    match gloo_net::http::Request::get("/config.json").send().await {
        Ok(resp) => resp.json::<RuntimeConfig>().await.unwrap_or_default(),
        Err(_) => RuntimeConfig::default(),
    }
}
