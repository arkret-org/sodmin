use std::sync::Mutex;
use std::sync::OnceLock;

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

/// Reset only the in-memory `Config` snapshot. Storage cleanup is the
/// caller's responsibility — `api::auth::logout` removes specific
/// auth/session keys so user preferences (`language`, `theme`,
/// `coauth_public_url`) survive logout.
pub fn clear_config() {
    *config_mutex().lock().unwrap() = Config::default();
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

/// Why `/config.json` could not be turned into a usable runtime config.
/// Surfaced verbatim by the login page so deployment operators see the
/// concrete failure (HTTP, JSON shape, missing field) instead of an
/// empty form that silently fails on the OAuth redirect.
#[derive(Debug, Clone)]
pub enum ConfigLoadError {
    /// `/config.json` did not respond (network error, 404, 5xx).
    Fetch(String),
    /// The response body was not valid JSON or missed required fields.
    Parse(String),
    /// The required `coauth_public_url` field was absent or empty.
    MissingCoauthUrl,
}

impl std::fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(msg) => write!(f, "Failed to fetch /config.json: {msg}"),
            Self::Parse(msg) => write!(f, "Failed to parse /config.json: {msg}"),
            Self::MissingCoauthUrl => {
                f.write_str("/config.json is missing the required `coauth_public_url` field")
            }
        }
    }
}

pub async fn load_runtime_config() -> Result<RuntimeConfig, ConfigLoadError> {
    let response = gloo_net::http::Request::get("/config.json")
        .send()
        .await
        .map_err(|e| ConfigLoadError::Fetch(e.to_string()))?;

    if !response.ok() {
        return Err(ConfigLoadError::Fetch(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let cfg = response
        .json::<RuntimeConfig>()
        .await
        .map_err(|e| ConfigLoadError::Parse(e.to_string()))?;

    if cfg.coauth_public_url.trim().is_empty() {
        return Err(ConfigLoadError::MissingCoauthUrl);
    }

    Ok(cfg)
}
