/// This module contains the main functionality of the `xdiff` library.
pub mod cli;
pub mod config;
pub mod utils;

pub use config::{
    is_default, DiffConfig, DiffProfile, LoadYaml, RequestProfile, ResponseExt, ResponseProfile,
    ValidateConfig,
};

/// Represents additional arguments for the `xdiff` library.
#[derive(Debug, Default)]
pub struct ExtraArgs {
    headers: Vec<(String, String)>,
    body: Vec<(String, String)>,
    query: Vec<(String, String)>,
}
