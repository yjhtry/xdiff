use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod xdiff;
pub mod xreq;

pub use xdiff::*;
pub use xreq::*;

use std::fmt::Write;
use std::str::FromStr;

/// Represents a request profile.
use anyhow::Result;
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response,
};
use serde_json::json;
use tokio::fs;
use url::Url;

use crate::ExtraArgs;

#[async_trait]
pub trait LoadYaml: Sized + ValidateConfig + DeserializeOwned {
    async fn load_yaml(path: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    fn from_yaml(content: &str) -> Result<Self> {
        let profiles: Self = serde_yaml::from_str(content)?;

        profiles.validate()?;
        Ok(profiles)
    }
}

pub trait ValidateConfig {
    fn validate(&self) -> Result<()>;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestProfile {
    pub url: Url,
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub body: Option<serde_json::Value>,
    #[serde(
        with = "http_serde::header_map",
        skip_serializing_if = "HeaderMap::is_empty",
        default
    )]
    pub headers: HeaderMap,
}

impl ValidateConfig for RequestProfile {
    fn validate(&self) -> Result<()> {
        if let Some(params) = &self.params {
            if !params.is_object() {
                return Err(anyhow::anyhow!(
                    "Params must be an object but got \n{}",
                    serde_yaml::to_string(params)?
                ));
            }
        }

        if let Some(body) = &self.body {
            if !body.is_object() {
                return Err(anyhow::anyhow!(
                    "Body must be an object. \n{}",
                    serde_yaml::to_string(body)?
                ));
            }
        }

        Ok(())
    }
}

impl RequestProfile {
    pub fn new(
        url: Url,
        method: Method,
        params: Option<serde_json::Value>,
        body: Option<serde_json::Value>,
        headers: HeaderMap,
    ) -> Self {
        RequestProfile {
            url,
            method,
            params,
            body,
            headers,
        }
    }

    pub async fn send(&self, extra: &ExtraArgs) -> Result<ResponseExt> {
        let (headers, body, query) = self.generate(extra)?;

        let client = Client::new();

        let request = client
            .request(self.method.clone(), self.url.clone())
            .headers(headers)
            .body(body)
            .query(&query)
            .build()?;

        let res = client.execute(request).await?;

        Ok(ResponseExt(res))
    }

    pub fn generate(&self, extra: &ExtraArgs) -> Result<(HeaderMap, String, serde_json::Value)> {
        let mut headers: HeaderMap = self.headers.clone();
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));

        if headers.get("content-type").is_none() {
            headers.insert(
                HeaderName::from_str("content-type")?,
                HeaderValue::from_str("application/json")?,
            );
        }

        for (key, value) in &extra.headers {
            headers.insert(HeaderName::from_str(key)?, HeaderValue::from_str(value)?);
        }

        for (key, value) in &extra.body {
            body[key] = json!(value);
        }

        for (key, value) in &extra.query {
            query[key] = json!(value);
        }

        let content_type = headers.get("content-type").unwrap();

        match content_type.to_str()? {
            "application/json" => {
                body = serde_json::to_value(&body)?;
                Ok((headers, body.to_string(), query))
            }
            "application/x-www-form-urlencoded" => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, body, query))
            }

            _ => panic!("Unsupported content type: {:?}", content_type),
        }
    }
}

fn empty_json_value(v: &Option<serde_json::Value>) -> bool {
    v.as_ref()
        .map_or(true, |v| v.is_null() || v.as_object().unwrap().is_empty())
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        let qs = url.query_pairs().collect::<Vec<_>>();

        let mut params = json!({});
        for (key, value) in qs {
            params[&*key] = json!(value);
        }

        Ok(RequestProfile::new(
            url,
            Method::GET,
            Some(params),
            None,
            HeaderMap::new(),
        ))
    }
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl ResponseExt {
    pub async fn get_text(self, skip_headers: &[String], skip_body: &[String]) -> Result<String> {
        let mut output = self.get_header_text(skip_headers)?;
        let is_json_content_type = self
            .0
            .headers()
            .get("content-type")
            .map_or(false, |v| v.to_str().unwrap().contains("application/json"));

        if !is_json_content_type {
            let text = self.0.text().await?;
            writeln!(&mut output, "{}", text)?;

            Ok(output)
        } else {
            let text = self.0.text().await?;
            let mut body = serde_json::from_str::<serde_json::Value>(&text)?;

            if let Some(mut_body) = body.as_object_mut() {
                for key in skip_body {
                    mut_body.remove(key);
                }
            }

            writeln!(&mut output, "{}", &serde_json::to_string_pretty(&body)?)?;

            Ok(output)
        }
    }

    pub fn get_header_text(&self, skip_headers: &[String]) -> Result<String> {
        let mut output = String::new();
        let headers = self.0.headers().clone();

        writeln!(&mut output, "{:?} {}", self.0.version(), self.0.status())?;

        for key in headers.keys() {
            if !skip_headers.contains(&key.to_string()) {
                writeln!(&mut output, "{}: {}", key, headers[key].to_str()?)?;
            }
        }

        writeln!(&mut output)?;

        Ok(output)
    }

    pub fn get_headers(&self) -> Vec<String> {
        self.0
            .headers()
            .keys()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
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

pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
