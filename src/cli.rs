use anyhow::{anyhow, Result};

use crate::ExtraArgs;

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
