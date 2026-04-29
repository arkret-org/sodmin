use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppletRegistration {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub as_token: String,
    #[serde(default)]
    pub hs_token: String,
    #[serde(default)]
    pub sender_localpart: String,
    #[serde(default)]
    pub rate_limited: bool,
    #[serde(default)]
    pub protocols: Vec<String>,
    #[serde(default)]
    pub namespaces: AppletNamespaces,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppletNamespaces {
    #[serde(default)]
    pub users: Vec<AppletNamespace>,
    #[serde(default)]
    pub aliases: Vec<AppletNamespace>,
    #[serde(default)]
    pub rooms: Vec<AppletNamespace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppletNamespace {
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub regex: String,
}
