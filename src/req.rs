use std::str::FromStr;

/// Represents a request profile.
use anyhow::Result;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::ExtraArgs;

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

impl RequestProfile {
    pub async fn send(&self, extra: &ExtraArgs) -> Result<()> {
        let (headers, body, query) = self.generate(&extra).await?;

        let client = Client::new();

        let request = client
            .request(self.method.clone(), self.url.clone())
            .headers(self.headers.clone())
            .query(&extra.query)
            .build()?;

        let res = client.execute(request).await?;

        let text = res.text().await?;
        println!("{:#?}", text);

        Ok(())
    }

    pub async fn generate<'a>(
        &self,
        extra: &'a ExtraArgs,
    ) -> Result<(HeaderMap, serde_json::Value, serde_json::Value)> {
        let mut headers: HeaderMap = self.headers.clone();
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));

        for (key, value) in &extra.headers {
            headers.insert(HeaderName::from_str(key)?, HeaderValue::from_str(value)?);
        }

        for (key, value) in &extra.body {
            body[key] = json!(value);
        }

        for (key, value) in &extra.query {
            query[key] = json!(value);
        }

        println!("{:#?}", query);

        Ok((headers, body, query))
    }
}
