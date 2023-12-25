use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

use crate::{utils::text_diff, ExtraArgs, RequestProfile};

/// Represents the configuration for performing diffs.
#[derive(Debug, Deserialize, Serialize)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl DiffConfig {
    pub fn new(profiles: HashMap<String, DiffProfile>) -> Self {
        Self { profiles }
    }
    pub async fn load_yaml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self> {
        let profiles: Self = serde_yaml::from_str(content)?;

        profiles.validate()?;
        Ok(profiles)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }

        Ok(())
    }
}

/// Represents a diff profile.
#[derive(Debug, Deserialize, Serialize)]
pub struct DiffProfile {
    pub request1: RequestProfile,
    pub request2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub response: ResponseProfile,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

impl DiffProfile {
    pub fn new(
        request1: RequestProfile,
        request2: RequestProfile,
        skip_headers: Vec<String>,
    ) -> Self {
        Self {
            request1,
            request2,
            response: ResponseProfile::new(skip_headers, vec![]),
        }
    }
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.request1.send(&args).await?;
        let res2 = self.request2.send(&args).await?;

        let text1 = res1
            .get_text(&self.response.skip_headers, &self.response.skip_body)
            .await?;
        let text2 = res2
            .get_text(&self.response.skip_headers, &self.response.skip_body)
            .await?;

        text_diff(text1, text2)
    }

    pub fn validate(&self) -> Result<()> {
        self.request1
            .validate()
            .context("request1 failed to validate")?;
        self.request2
            .validate()
            .context("request2 failed to validate")?;

        Ok(())
    }
}

/// Represents a response profile.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}
