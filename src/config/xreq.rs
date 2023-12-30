use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{is_default, LoadYaml, RequestProfile, ResponseProfile, ValidateConfig};

/// Represents the configuration for performing diffs.
#[derive(Debug, Deserialize, Serialize)]
pub struct ReqConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, ReqProfile>,
}
impl LoadYaml for ReqConfig {}

impl ValidateConfig for ReqConfig {
    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }

        Ok(())
    }
}

impl ReqConfig {
    pub fn new(profiles: HashMap<String, ReqProfile>) -> Self {
        Self { profiles }
    }

    pub fn get_profile(&self, name: &str) -> Option<&ReqProfile> {
        self.profiles.get(name)
    }
}

/// Represents a diff profile.
#[derive(Debug, Deserialize, Serialize)]
pub struct ReqProfile {
    pub request: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub response: ResponseProfile,
}

impl ValidateConfig for ReqProfile {
    fn validate(&self) -> Result<()> {
        self.request
            .validate()
            .context("request failed to validate")?;

        Ok(())
    }
}

impl ReqProfile {
    pub fn new(request: RequestProfile, skip_headers: Vec<String>) -> Self {
        Self {
            request,
            response: ResponseProfile::new(skip_headers, vec![]),
        }
    }
}
