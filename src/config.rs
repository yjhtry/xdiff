use anyhow::Result;
use http::{HeaderMap, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use url::Url;

use crate::ExtraArgs;

/// Represents the configuration for performing diffs.
#[derive(Debug, Deserialize, Serialize)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl DiffConfig {
    /// Loads the diff configuration from a YAML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML file.
    ///
    /// # Returns
    ///
    /// The loaded `DiffConfig` if successful, otherwise an `anyhow::Error`.
    pub async fn load_yaml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_str(&content)
    }

    /// Creates a `DiffConfig` from a YAML string.
    ///
    /// # Arguments
    ///
    /// * `content` - The YAML string representing the diff configuration.
    ///
    /// # Returns
    ///
    /// The created `DiffConfig` if successful, otherwise an `anyhow::Error`.
    pub fn from_str(content: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }

    /// Retrieves a diff profile by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the diff profile.
    ///
    /// # Returns
    ///
    /// The diff profile if found, otherwise `None`.
    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

/// Represents a diff profile.
#[derive(Debug, Deserialize, Serialize)]
pub struct DiffProfile {
    pub request1: RequestProfile,
    pub request2: RequestProfile,
    pub response: ResponseProfile,
}

impl DiffProfile {
    /// Performs a diff operation using the specified arguments.
    ///
    /// # Arguments
    ///
    /// * `_args` - The diff arguments.
    ///
    /// # Returns
    ///
    /// The diff result as a string if successful, otherwise an `anyhow::Error`.
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        println!("{:#?}", self);
        println!("{:#?}", args);

        Ok("".to_string())
    }
}

/// Represents a request profile.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestProfile {
    pub url: Url,
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
    #[serde(
        with = "http_serde::header_map",
        skip_serializing_if = "HeaderMap::is_empty",
        default
    )]
    pub headers: HeaderMap,
}

/// Represents a response profile.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}
