use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    is_default, utils::text_diff, ExtraArgs, LoadYaml, RequestProfile, ResponseProfile,
    ValidateConfig,
};

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
    pub request1: RequestProfile,
    pub request2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub response: ResponseProfile,
}

impl ValidateConfig for ReqProfile {
    fn validate(&self) -> Result<()> {
        self.request1
            .validate()
            .context("request1 failed to validate")?;
        self.request2
            .validate()
            .context("request2 failed to validate")?;

        Ok(())
    }
}

impl ReqProfile {
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
}
