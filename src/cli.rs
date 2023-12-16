use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::ExtraArgs;

/// Diff two HTTP requests and compare the differences of their responses.
#[derive(Parser, Debug, Clone)]
#[clap(version, author, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
#[non_exhaustive]
pub enum Action {
    Run(RunArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args. Could be used to voerride the query, headers and body of the request.
    /// for query params, use `-e key=value`
    /// for headers, use `-e %key=value`
    /// for body, use `-e @key=value`
    #[clap(short, long, value_parser = parse_key_val, number_of_values=1)]
    pub extra_params: Vec<KeyVal>,
    /// config file
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

/// key value pair
#[derive(Debug, Clone)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    val: String,
}

pub fn parse_key_val<'a>(s: &'a str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');
    let retrieve = |v: Option<&'a str>| -> Result<&'a str> {
        Ok(v.ok_or_else(|| anyhow!("Invalid key value pair: {}", s))?
            .trim())
    };
    let key = retrieve(parts.next())?;
    let val = retrieve(parts.next())?;

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, key[1..].to_string()),
        Some('@') => (KeyValType::Body, key[1..].to_string()),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key.to_string()),
        _ => anyhow::bail!("Invalid key value pair: {}", s),
    };

    Ok(KeyVal {
        key_type,
        key,
        val: val.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(v: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut body = vec![];
        let mut query = vec![];

        for kv in v {
            match kv.key_type {
                KeyValType::Header => headers.push((kv.key, kv.val)),
                KeyValType::Body => body.push((kv.key, kv.val)),
                KeyValType::Query => query.push((kv.key, kv.val)),
            }
        }

        Self {
            headers,
            body,
            query,
        }
    }
}
